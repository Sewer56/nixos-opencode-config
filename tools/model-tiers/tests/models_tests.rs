use opencode_model_tiers::models;

#[test]
fn test_parse_models_output_dedupes_and_ignores_noise() {
    let output =
        "\nnot-a-model\nsewer-axonhub/GLM-5.1\nsewer-axonhub/GLM-5.1\nsewer-axonhub-work/gpt-5.5\n";
    let models = models::parse_models_output(output).unwrap();
    assert_eq!(
        models,
        vec![
            "sewer-axonhub/GLM-5.1".to_string(),
            "sewer-axonhub-work/gpt-5.5".to_string(),
        ]
    );
}

#[test]
fn test_parse_models_output_rejects_empty_when_no_valid_models() {
    let output = "\nnot-a-model\nno-slash\nanother-bad\n";
    let result = models::parse_models_output(output);
    assert!(result.is_err());
}

#[test]
fn test_filter_models_uses_tokens_and_work_provider() {
    let models = vec![
        "sewer-axonhub/GLM-5.1".to_string(),
        "sewer-axonhub/gpt-5.5".to_string(),
        "sewer-axonhub-work/gpt-5.5".to_string(),
        "sewer-axonhub-work/gpt-5.4-mini".to_string(),
    ];

    let got = models::filter_models("normal", &models, "gpt 5.5");
    assert_eq!(
        got,
        vec![
            "sewer-axonhub/gpt-5.5".to_string(),
            "sewer-axonhub-work/gpt-5.5".to_string(),
        ]
    );

    let got = models::filter_models("work", &models, "gpt");
    assert_eq!(
        got,
        vec![
            "sewer-axonhub-work/gpt-5.5".to_string(),
            "sewer-axonhub-work/gpt-5.4-mini".to_string(),
        ]
    );
}

#[test]
fn test_filter_models_case_insensitive() {
    let models = vec![
        "sewer-axonhub/GPT-5.5".to_string(),
        "sewer-axonhub/glm-5.1".to_string(),
    ];

    let got = models::filter_models("normal", &models, "Gpt");
    assert_eq!(got, vec!["sewer-axonhub/GPT-5.5".to_string()]);
}

#[test]
fn test_filter_models_empty_query_returns_all() {
    let models = vec![
        "sewer-axonhub/a".to_string(),
        "sewer-axonhub-work/b".to_string(),
    ];

    let got = models::filter_models("normal", &models, "");
    assert_eq!(got.len(), 2);
}

#[test]
fn test_filter_models_work_profile_filters_non_work() {
    let models = vec![
        "sewer-axonhub/a".to_string(),
        "sewer-axonhub-work/b".to_string(),
    ];

    let got = models::filter_models("work", &models, "");
    assert_eq!(got, vec!["sewer-axonhub-work/b".to_string()]);
}
