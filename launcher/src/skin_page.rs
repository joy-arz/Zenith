//! Local skin and cape controls with a draggable player preview.
use super::*;
use crate::wardrobe_page::{plain_skin, viewer};
use gpui::RenderImage;
use horizon_cosmetics::{preview::process_legacy_skin, Texture};
use horizon_launcher::skins::{self, Model};
use std::path::PathBuf;
use std::sync::Arc;

/// A baked preview, or None without one.
type Preview = Option<Arc<RenderImage>>;

#[derive(Default)]
pub(super) struct SkinPanel {
    /// The offline name it is for.
    pub(super) name: String,
    seen: Option<(f32, Arc<Texture>, Preview)>,
    viewer: Option<gpui::Entity<viewer::Viewer>>,
    shown: Option<viewer::Outfit>,
    dragging: bool,
    extruded: bool,
    /// What the last choice did, or why not.
    note: Option<Result<String, String>>,
}

/// Points per skin texel in the preview.
const TEXEL: f32 = 6.;

impl Launcher {
    fn select_skin(&mut self, name: String) {
        if self.skin.as_ref().is_none_or(|panel| panel.name != name) {
            self.skin = Some(SkinPanel {
                name,
                ..SkinPanel::default()
            });
        }
    }

    pub(super) fn skins_page(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let names = self.accounts.offline.clone();
        let page = div().flex().flex_col().gap(px(16.)).child(page_header(
            "Skins",
            "Upload a skin for an offline profile and preview it in 3D.",
            &t,
        ));
        if names.is_empty() {
            self.skin = None;
            return page.child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(10.))
                    .child(text(theme::UI).text_color(t.muted).child(tr(
                        "Add an offline profile in Manage accounts to upload a skin.",
                    )))
                    .child(
                        button(
                            "skin-manage-accounts",
                            "Manage accounts",
                            Kind::Secondary,
                            &t,
                        )
                        .on_click(cx.listener(
                            |this, _: &ClickEvent, _, cx| {
                                this.manage_accounts();
                                cx.notify();
                            },
                        )),
                    ),
            );
        }
        if !self
            .skin
            .as_ref()
            .is_some_and(|panel| names.contains(&panel.name))
        {
            let selected = match self.accounts.player() {
                account::Player::Offline(name) if names.contains(&name) => name,
                _ => names[0].clone(),
            };
            self.select_skin(selected);
        }
        let profiles = names.into_iter().enumerate().map(|(i, name)| {
            let active = self.skin.as_ref().is_some_and(|panel| panel.name == name);
            button(
                format!("skin-profile-{i}"),
                name.clone(),
                if active { Kind::Secondary } else { Kind::Ghost },
                &t,
            )
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.select_skin(name.clone());
                cx.notify();
            }))
        });
        let selection = div().flex().flex_wrap().gap(px(6.)).children(profiles);
        page.child(selection).children(self.skin_panel(cx))
    }

    fn save_look(&mut self, change: impl FnOnce(&mut skins::Look)) -> Result<(), String> {
        let Some(name) = self.skin.as_ref().map(|p| p.name.clone()) else {
            return Ok(());
        };
        let mut accounts = Accounts::load();
        // The name may have been forgotten while a file dialog was open: keep nothing for it.
        if !accounts
            .offline
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&name))
        {
            return Ok(());
        }
        let mut look = accounts.look(&name);
        change(&mut look);
        accounts.looks.insert(name.to_ascii_lowercase(), look);
        accounts.save()?;
        self.accounts = accounts;
        if let Some(p) = &mut self.skin {
            p.seen = None;
        }
        Ok(())
    }

    fn choose_look_file(&mut self, cape: bool, cx: &mut Context<Self>) {
        let Some(name) = self.skin.as_ref().map(|panel| panel.name.clone()) else {
            return;
        };
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr("Choose").into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.pop() else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                if this.skin.as_ref().is_some_and(|panel| panel.name == name) {
                    this.take_look_file(path, cape);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn take_look_file(&mut self, path: PathBuf, cape: bool) {
        let Some(name) = self.skin.as_ref().map(|p| p.name.clone()) else {
            return;
        };
        // The name may have been forgotten while the file dialog was open.
        if !Accounts::load()
            .offline
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&name))
        {
            return;
        }
        let done = skins::keep(&skins::dir(), &name, &path, cape).and_then(|()| {
            self.save_look(|l| match cape {
                false => l.skin = true,
                true => l.cape = true,
            })
        });
        if let Some(p) = &mut self.skin {
            let said = match cape {
                false => trf("{name}'s skin is kept.", &[("name", &name)]),
                true => trf("{name}'s cape is kept.", &[("name", &name)]),
            };
            p.note = Some(done.map(|()| said));
        }
    }

    fn previews(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<(gpui::Entity<viewer::Viewer>, bool, Preview)> {
        let scale = self.scale;
        let panel = self.skin.as_mut()?;
        let look = self.accounts.look(&panel.name);
        if panel
            .seen
            .as_ref()
            .is_some_and(|(previous, ..)| *previous != scale)
        {
            panel.seen = None;
        }
        let (_, skin, cape) = panel.seen.get_or_insert_with(|| {
            let (skin_file, cape_file) = skins::files(&skins::dir(), &panel.name);
            let skin = std::fs::read(skin_file)
                .ok()
                .filter(|_| look.skin)
                .and_then(|bytes| skins::read_skin(&bytes).ok())
                .and_then(|skin| process_legacy_skin(skin.width(), skin.height(), skin.as_raw()))
                .map(|rgba| {
                    Arc::new(Texture {
                        width: 64,
                        height: 64,
                        rgba,
                    })
                })
                .unwrap_or_else(plain_skin);
            let cape = std::fs::read(cape_file)
                .ok()
                .filter(|_| look.cape)
                .and_then(|bytes| skins::read_cape(&bytes).ok())
                .map(|png| art::bake(&skins::cape_front(&png), 10, 16, TEXEL, scale));
            (scale, skin, cape)
        });
        let (skin, cape) = (skin.clone(), cape.clone());
        let viewer = panel
            .viewer
            .get_or_insert_with(|| cx.new(|_| viewer::Viewer::new()))
            .clone();
        let outfit = viewer::Outfit {
            skin,
            slim: look.model == Model::Slim,
            extruded: panel.extruded,
            items: Vec::new(),
        };
        if panel.shown.as_ref() != Some(&outfit) {
            panel.shown = Some(outfit.clone());
            let to = viewer.clone();
            cx.defer(move |cx| to.update(cx, |view, cx| view.dress(outfit, false, cx)));
        }
        Some((viewer, panel.dragging, cape))
    }

    fn skin_panel(&mut self, cx: &mut Context<Self>) -> Option<Div> {
        let t = self.theme;
        let name = self.skin.as_ref()?.name.clone();
        let look = self.accounts.look(&name);
        let (viewer, dragging, cape) = self.previews(cx)?;
        let weak = cx.entity().downgrade();
        let preview = viewer::frame(
            viewer.clone(),
            (viewer::W, viewer::H),
            dragging,
            move |on, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if let Some(panel) = this.skin.as_mut() {
                        panel.dragging = on;
                        cx.notify();
                    }
                });
            },
        );
        let reset = button("skin-reset-view", "Reset view", Kind::Ghost, &t)
            .on_click(move |_: &ClickEvent, _, cx| viewer.update(cx, |view, cx| view.reset(cx)));
        let extruded = self.skin.as_ref()?.extruded;
        let layers = button(
            "skin-layer-depth",
            if extruded {
                "3D skin layers on"
            } else {
                "3D skin layers off"
            },
            Kind::Secondary,
            &t,
        )
        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
            if let Some(panel) = this.skin.as_mut() {
                panel.extruded = !panel.extruded;
                cx.notify();
            }
        }));
        let weak = cx.entity().downgrade();
        let model = segments(
            "skin-model",
            &[(Model::Classic, "Classic arms"), (Model::Slim, "Slim arms")],
            look.model,
            &t,
            move |m, _, cx| {
                let _ = weak.update(cx, |this, cx| {
                    let _ = this.save_look(|l| l.model = m);
                    cx.notify();
                });
            },
        );
        let pick = |id: &'static str, label: &'static str, cape: bool, cx: &mut Context<Self>| {
            button(id, label, Kind::Secondary, &t).on_click(cx.listener(
                move |this, _: &ClickEvent, _, cx| {
                    this.choose_look_file(cape, cx);
                    cx.notify();
                },
            ))
        };
        let drop = |id: &'static str, label: &'static str, cape: bool, cx: &mut Context<Self>| {
            button(id, label, Kind::Ghost, &t).on_click(cx.listener(
                move |this, _: &ClickEvent, _, cx| {
                    let _ = this.save_look(|l| match cape {
                        false => l.skin = false,
                        true => l.cape = false,
                    });
                    cx.notify();
                },
            ))
        };
        let mut skin_buttons = div().flex().flex_wrap().gap(px(6.)).child(pick(
            "skin-choose",
            "Upload skin…",
            false,
            cx,
        ));
        if look.skin {
            skin_buttons = skin_buttons.child(drop("skin-remove", "Default skin", false, cx));
        }
        let mut cape_buttons = div().flex().flex_wrap().gap(px(6.)).child(pick(
            "cape-choose",
            "Upload cape…",
            true,
            cx,
        ));
        if look.cape {
            cape_buttons = cape_buttons.child(drop("cape-remove", "No cape", true, cx));
        }
        let note = self.skin.as_ref().and_then(|p| p.note.clone());
        Some(
            plate(&t)
                .flex()
                .flex_col()
                .gap(px(12.))
                .px(px(16.))
                .py(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(16.))
                        .child(div().flex().flex_col().items_center().flex_none()
                            .bg(t.well).child(preview)
                            .child(text(theme::CAPTION).text_color(t.muted)
                                .child(tr("Drag to turn, scroll to zoom.")))
                            .child(reset))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_start()
                                .flex_1()
                                .min_w(px(200.))
                                .gap(px(10.))
                                .child(
                                    caps(trf("{name}'s skin", &[("name", &name)]))
                                        .text_color(t.muted),
                                )
                                .child(skin_buttons)
                                .child(model)
                                .child(layers)
                                .child(cape_buttons)
                                .children(cape.map(|image| gpui::img(image).w(px(10. * TEXEL)).h(px(16. * TEXEL))))
                                .children(note.map(|n| match n {
                                    Ok(said) => {
                                        text(theme::CAPTION).text_color(t.muted).child(said)
                                    }
                                    Err(e) => text(theme::CAPTION).text_color(t.danger).child(e),
                                })),
                        ),
                )
                .child(
                    text(theme::CAPTION)
                        .text_color(t.muted)
                        .line_height(px(16.))
                        .child(tr(
                            "A 64×64 (or 64×32) skin PNG and a 64×32 cape PNG, kept by the \
                     launcher and used in game when this offline name plays. Choose \
                     Classic or Slim arms to match your PNG. Restart the game after changing a skin.",
                        )),
                ),
        )
    }
}
