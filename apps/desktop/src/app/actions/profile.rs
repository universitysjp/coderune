//! The developer baseline form, its draft, and onboarding confirmation.

use crate::app::*;

impl CoderuneApp {
    /// Whether the developer baseline is still missing, so the onboarding dialog
    /// stays in front of the workspace.
    pub(crate) fn onboarding_required(&self) -> bool {
        self.workspace
            .as_ref()
            .is_some_and(|workspace| !workspace.onboarding.can_start_project)
    }

    fn profile_base(&self) -> ProfileDraft {
        self.workspace
            .as_ref()
            .map(|workspace| {
                workspace
                    .onboarding
                    .profile
                    .as_ref()
                    .map(ProfileDraft::from_profile)
                    .or_else(|| {
                        workspace
                            .onboarding
                            .draft
                            .as_ref()
                            .map(|draft| draft.draft.clone())
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub(crate) fn refresh_profile_form(&mut self) {
        if let Some(workspace) = self.workspace.as_ref() {
            self.profile_form = ProfileForm::from_value(
                workspace.onboarding.profile.as_ref(),
                workspace.onboarding.draft.as_ref(),
            );
        }
    }

    pub(crate) fn save_draft(&mut self) {
        if self.persist_draft() {
            self.notice = Some("Progress saved. You can close and resume.".to_string());
        }
    }

    /// Save the current form as the resumable draft. Returns whether it stuck.
    fn persist_draft(&mut self) -> bool {
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return false;
        };
        match workspace.save_onboarding_draft(draft) {
            Ok(()) => {
                self.error = None;
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    /// Why the onboarding dialog cannot be confirmed, if it cannot. Only the two
    /// questions every project needs are required; the rest can wait in Profile.
    fn onboarding_blocker(&self) -> Option<&'static str> {
        if self.profile_form.display_name.text.trim().is_empty() {
            Some("Add a display name or pseudonym to continue.")
        } else if self.profile_form.stage.is_none() {
            Some("Choose the experience stage that fits you today.")
        } else {
            None
        }
    }

    pub(crate) fn confirm_onboarding(&mut self) {
        if let Some(message) = self.onboarding_blocker() {
            self.error = Some(AppError::new(
                coderune_domain::ErrorCode::Validation,
                message,
            ));
            self.notice = None;
            return;
        }
        self.confirm_profile();
    }

    /// Closes the onboarding dialog and leaves the same answers in the Profile
    /// panel, where the rest of the baseline can be finished.
    pub(crate) fn dismiss_onboarding(&mut self) {
        self.onboarding_visible = false;
        self.view = View::Profile;
        self.profile_tab = ProfileTab::Baseline;
        self.error = None;
        self.notice = None;
        self.show_workspace();
    }

    pub(crate) fn open_profile_details(&mut self) {
        self.view = View::Profile;
        self.profile_tab = ProfileTab::Details;
        self.error = None;
        self.notice = None;
    }

    pub(crate) fn toggle_profile_section(&mut self, section: ProfileSection) {
        self.profile_section = (self.profile_section != Some(section)).then_some(section);
    }

    pub(crate) fn confirm_profile(&mut self) {
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.confirm_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!(
                    "Developer baseline v{} confirmed. Add more to your profile whenever you want.",
                    profile.version
                ));
                self.error = None;
                self.onboarding_visible = false;
                self.view = View::Home;
                self.profile_tab = ProfileTab::Baseline;
                self.profile_section = None;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn save_profile(&mut self) {
        // Before the first baseline is confirmed, the Profile panel is the
        // onboarding fallback: the same form, with the same confirmation.
        if self.onboarding_required() {
            self.confirm_profile();
            return;
        }
        let base = self.profile_base();
        let draft = self.profile_form.to_draft(base);
        let Some(workspace) = self.workspace.as_mut() else {
            return;
        };
        match workspace.update_profile(draft) {
            Ok(profile) => {
                self.notice = Some(format!("Profile v{} saved.", profile.version));
                self.error = None;
                self.refresh_profile_form();
            }
            Err(error) => self.error = Some(error),
        }
    }
}
