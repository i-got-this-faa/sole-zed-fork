use std::sync::Arc;

use gpui::{App, AppContext as _};
use workspace::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitStep {
    pub crate_name: &'static str,
    pub responsibility: &'static str,
}

pub const CORE_DIRECT_DEPENDENCIES: &[&str] = &[
    "command_palette",
    "editor",
    "gpui",
    "gpui_tokio",
    "outline_panel",
    "release_channel",
    "search",
    "semver",
    "theme",
    "theme_settings",
    "workspace",
];

pub const PROJECT_UI_DIRECT_DEPENDENCIES: &[&str] = &["file_finder", "git_ui", "project_panel"];

pub const OMITTED_PRODUCT_SURFACES: &[&str] = &[
    "agent",
    "agent_ui",
    "ai_onboarding",
    "audio",
    "auto_update",
    "call",
    "channel",
    "client UI ownership",
    "cloud_llm_client",
    "collab",
    "collab_ui",
    "crashes",
    "feedback",
    "image_viewer",
    "onboarding",
    "telemetry app setup",
    "terminal_view",
    "zed",
];

pub const INIT_STEPS: &[InitStep] = &[
    InitStep {
        crate_name: "gpui_tokio",
        responsibility: "attach async executor support to the GPUI app",
    },
    InitStep {
        crate_name: "theme_settings",
        responsibility: "load base theme and editor theme settings",
    },
    InitStep {
        crate_name: "workspace",
        responsibility: "register workspace model, panes, panels, and global workspace actions",
    },
    InitStep {
        crate_name: "release_channel",
        responsibility: "provide version metadata expected by editor/workspace UI",
    },
    InitStep {
        crate_name: "command_palette",
        responsibility: "register command palette integration for workspace actions",
    },
    InitStep {
        crate_name: "editor",
        responsibility: "register the Zed editor item and editor actions",
    },
    InitStep {
        crate_name: "outline_panel",
        responsibility: "register the document outline panel",
    },
    InitStep {
        crate_name: "search",
        responsibility: "register buffer and project search",
    },
];

pub const PROJECT_UI_INIT_STEPS: &[InitStep] = &[
    InitStep {
        crate_name: "git_ui",
        responsibility: "register editor/workspace git UI affordances",
    },
    InitStep {
        crate_name: "file_finder",
        responsibility: "register file and path picker actions",
    },
    InitStep {
        crate_name: "project_panel",
        responsibility: "register the project tree panel",
    },
];

pub fn init(app_state: Arc<AppState>, cx: &mut App) {
    gpui_tokio::init(cx);
    theme_settings::init(theme::LoadThemes::JustBase, cx);
    workspace::init(app_state, cx);
    release_channel::init(semver::Version::new(0, 0, 0), cx);
    command_palette::init(cx);
    editor::init(cx);
    outline_panel::init(cx);
    search::init(cx);
    register_buffer_search_callbacks(cx);
    init_project_ui(cx);
}

#[cfg(feature = "project-ui")]
fn init_project_ui(cx: &mut App) {
    git_ui::init(cx);
    file_finder::init(cx);
    project_panel::init(cx);
}

#[cfg(not(feature = "project-ui"))]
fn init_project_ui(_cx: &mut App) {}

fn register_buffer_search_callbacks(cx: &mut App) {
    cx.set_global(workspace::PaneSearchBarCallbacks {
        setup_search_bar: |languages, toolbar, window, cx| {
            let search_bar = cx.new(|cx| search::BufferSearchBar::new(languages, window, cx));
            toolbar.update(cx, |toolbar, cx| {
                toolbar.add_item(search_bar, window, cx);
            });
        },
        wrap_div_with_search_actions: search::buffer_search::register_pane_search_actions,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_product_surfaces_are_not_direct_boundary_dependencies() {
        for omitted in OMITTED_PRODUCT_SURFACES {
            assert!(
                !CORE_DIRECT_DEPENDENCIES.contains(omitted),
                "{omitted} must stay outside the direct SOLE editor host boundary"
            );
        }
    }

    #[test]
    fn init_sequence_stays_editor_focused() {
        let step_names = INIT_STEPS
            .iter()
            .map(|step| step.crate_name)
            .collect::<Vec<_>>();

        assert!(step_names.contains(&"workspace"));
        assert!(step_names.contains(&"editor"));
        assert!(step_names.contains(&"outline_panel"));
        assert!(step_names.contains(&"search"));
        assert!(!step_names.contains(&"project_panel"));
        assert!(!step_names.contains(&"terminal_view"));
        assert!(!step_names.contains(&"agent_ui"));
        assert!(!step_names.contains(&"collab_ui"));
    }

    #[test]
    fn project_ui_surface_is_default_off() {
        let core_step_names = INIT_STEPS
            .iter()
            .map(|step| step.crate_name)
            .collect::<Vec<_>>();

        for dependency in PROJECT_UI_DIRECT_DEPENDENCIES {
            assert!(
                !CORE_DIRECT_DEPENDENCIES.contains(dependency),
                "{dependency} must stay out of the default core host boundary"
            );
        }

        for step in PROJECT_UI_INIT_STEPS {
            assert!(
                !core_step_names.contains(&step.crate_name),
                "{} must stay out of the default init sequence",
                step.crate_name
            );
        }
    }
}
