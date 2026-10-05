use super::*;

#[test]
fn ordinary_release_ignores_instance_and_optional_feature_does_not_change_default_owner() {
    let project = Path::new("project");
    for enabled in [false, true] {
        assert!(
            isolated(project, None, "cn.stagemaster.desktop", false, enabled)
                .unwrap()
                .is_none()
        );
    }
    assert!(
        isolated(
            project,
            Some(OsStr::new("../not-an-instance")),
            "cn.stagemaster.desktop",
            false,
            false
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn isolated_release_requires_capability_and_matching_internal_identity() {
    let project = Path::new("project");
    let instance = Some(OsStr::new("desktop-release-AbC1"));
    let identifier = "cn.stagemaster.acceptance.desktop-release-abc1";
    assert!(
        isolated(project, instance, identifier, false, false)
            .unwrap()
            .is_none()
    );
    let selected = isolated(project, instance, identifier, false, true)
        .unwrap()
        .expect("explicit internal release must have an isolated owner");
    assert_eq!(
        selected.data,
        project.join("tmp/desktop-desktop-release-AbC1")
    );
    assert!(selected.logs.starts_with(&selected.data));
    assert!(selected.temporary.starts_with(&selected.data));
}

#[test]
fn internal_release_never_falls_back_for_missing_or_invalid_instance() {
    let project = Path::new("project");
    let identifier = "cn.stagemaster.acceptance.desktop-release-abc1";
    for instance in [
        None,
        Some(OsStr::new("")),
        Some(OsStr::new("../other")),
        Some(OsStr::new("有空格")),
    ] {
        assert!(isolated(project, instance, identifier, false, true).is_err());
    }
}

#[test]
fn a_customer_or_different_internal_identity_cannot_use_release_override() {
    for identifier in [
        "cn.stagemaster.desktop",
        "cn.stagemaster.acceptance.other",
        "cn.stagemaster.acceptance.",
    ] {
        assert!(
            isolated(
                Path::new("project"),
                Some(OsStr::new("desktop-release-AbC1")),
                identifier,
                false,
                true
            )
            .is_err()
        );
    }
}

#[test]
fn debug_owners_keep_original_behavior_with_or_without_new_capability() {
    for enabled in [false, true] {
        let defaults = isolated(
            Path::new("project"),
            None,
            "cn.stagemaster.desktop",
            true,
            enabled,
        )
        .unwrap()
        .unwrap();
        assert_eq!(defaults.data, Path::new("project/data"));
        let acceptance = isolated(
            Path::new("project"),
            Some(OsStr::new("existing-009")),
            "cn.stagemaster.desktop",
            true,
            enabled,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            acceptance.data,
            Path::new("project/tmp/desktop-existing-009")
        );
    }
}
