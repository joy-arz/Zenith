# Horizon pull request queue

This snapshot covers all 450 open Horizon pull requests when the queue was collected. It records each GitHub PR, its exact head commit, and the result of a local Git merge simulation. See [the complete audit](pull-requests.csv).

| Revision | Commit |
| --- | --- |
| Previous Zenith | `373c70b04f1af874c19f0ad29b8cf390ef931a2a` |
| Horizon main | `1453e0e614af4b2d04dd0d228a94162b5d459910` |
| Zenith merge tested | `72fea9bd2e9fd4c321c64b092077669cab404688` |

**184 queued for review.** These PRs applied in the recorded sequence to the pinned Zenith revision. Review and test each batch before applying it to a release branch. If a PR head changes, repeat its checks and replay the affected queue.

**266 held.** The counts below can overlap because a pull request can have several reasons.

| Hold reason | PRs |
| --- | ---: |
| Conflicts with the pinned Zenith merge | 43 |
| Conflicts with an earlier queued change | 32 |
| GitHub reported a conflict with the PR's Horizon base | 26 |
| Touches a path Zenith changed | 219 |
| Touches build or workflow files | 38 |
| Targets another PR branch | 27 |
| Draft pull request | 20 |

Zenith's release builds use the merged Horizon main snapshot. The queue branch contains the audit, and the simulated PR merges are available locally under `refs/zenith-queue/simulation`.
