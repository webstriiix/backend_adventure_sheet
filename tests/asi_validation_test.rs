use adventure_sheets::models::character::AsiChoiceRequest;

#[test]
fn test_bump_single_1_ok() {
    let req = AsiChoiceRequest {
        bump_str: Some(1),
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_bump_single_2_ok() {
    let req = AsiChoiceRequest {
        bump_str: Some(2),
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_bump_single_3_invalid() {
    let req = AsiChoiceRequest {
        bump_str: Some(3),
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err, "Cannot increase str by more than 2");
}

#[test]
fn test_two_bumps_at_1_ok() {
    let req = AsiChoiceRequest {
        bump_str: Some(1),
        bump_dex: Some(1),
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_two_bumps_total_gt_2_invalid() {
    let req = AsiChoiceRequest {
        bump_str: Some(2),
        bump_dex: Some(1),
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(
        err,
        "When increasing two abilities, total increase cannot exceed 2"
    );
}

#[test]
fn test_feat_selected_with_bump_invalid() {
    let req = AsiChoiceRequest {
        bump_str: Some(1),
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: Some(1),
        source_type: None,
        gained_at_level: None,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err, "Cannot increase ability scores when selecting a feat");
}

#[test]
fn test_feat_selected_no_bump_ok() {
    let req = AsiChoiceRequest {
        bump_str: None,
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: Some(1),
        source_type: None,
        gained_at_level: None,
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_nothing_selected_invalid() {
    let req = AsiChoiceRequest {
        bump_str: None,
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(
        err,
        "Must increase at least one ability score or select a feat"
    );
}

#[test]
fn test_negative_bump_invalid() {
    let req = AsiChoiceRequest {
        bump_str: Some(-1),
        bump_dex: None,
        bump_con: None,
        bump_int: None,
        bump_wis: None,
        bump_cha: None,
        feat_id: None,
        source_type: None,
        gained_at_level: None,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err, "Cannot decrease str");
}
