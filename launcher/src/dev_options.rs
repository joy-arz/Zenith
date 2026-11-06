//! Offline access is a visible Settings section in this fork. Minecraft resources still
//! come from the player's own install; downloads retain the Microsoft ownership check.
use super::*;

/// Clicks on the launcher's version that reveal the section.
const CLICKS: u8 = 7;

impl Launcher {
    /// A click on the sidebar's launcher version; the last one opens Settings on the section.
    pub(super) fn version_clicked(&mut self) {
        if self.dev_options {
            return;
        }
        self.dev_clicks += 1;
        if self.dev_clicks >= CLICKS {
            self.dev_options = true;
            self.go(Page::Settings);
        }
    }

    /// The section, when revealed or when one of its options is on.
    pub(super) fn dev_options_section(&self, cx: &mut Context<Self>) -> Option<Div> {
        let t = self.theme;
        let toggle = switch("offline-access", self.settings.offline_enabled(), &t).on_click(
            cx.listener(|this, _: &ClickEvent, _, cx| {
                this.settings.offline_access = Some(!this.settings.offline_enabled());
                this.settings.dev_offline = false;
                this.save_settings();
                cx.notify();
            }),
        );
        Some(
            div()
                .flex()
                .flex_col()
                .child(
                    section_title("Offline access", "terminal", &t)
                        .mt(px(20.))
                        .mb(px(12.)),
                )
                .child(index_row(
                    0,
                    tr("Offline names without an account"),
                    Some(
                        div()
                            .child(tr(
                                "Offline names play without a Microsoft account that owns the game. \
                                 Minecraft still comes from your own install.",
                            ))
                            .into_any_element(),
                    ),
                    Some(toggle.into_any_element()),
                    &t,
                )),
        )
    }
}
