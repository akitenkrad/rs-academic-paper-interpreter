//! 統合テスト - APIキーを使用した実際のAPI呼び出しテスト
//!
//! 実行方法:
//! - 全統合テスト: `cargo test --test integration_tests -- --ignored`
//! - OpenAIのみ: `cargo test --test integration_tests openai -- --ignored`
//! - Anthropicのみ: `cargo test --test integration_tests anthropic -- --ignored`
//! - Ollamaのみ: `cargo test --test integration_tests ollama -- --ignored`
//!
//! 必要な環境変数:
//! - OPENAI_API_KEY: OpenAIテスト用
//! - ANTHROPIC_API_KEY: Anthropicテスト用
//! - OLLAMA_BASE_URL (オプション): Ollamaサーバーのベースurl（デフォルト: http://localhost:11434）

use academic_paper_interpreter::agents::{
    AnalysisAgent, AnthropicProvider, LlmConfig, LlmProvider, Message, OllamaProvider,
    OpenAiProvider, PaperAnalyzer,
};
use academic_paper_interpreter::models::{AcademicPaper, Author};

/// 文字列をUnicode文字境界で安全に切り詰める
fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// テスト用のサンプル論文データを作成
fn create_sample_paper() -> AcademicPaper {
    let mut paper = AcademicPaper::new();
    paper.title = "Attention Is All You Need".to_string();
    paper.abstract_text = r#"The dominant sequence transduction models are based on complex recurrent or convolutional neural networks that include an encoder and a decoder. The best performing models also connect the encoder and decoder through an attention mechanism. We propose a new simple network architecture, the Transformer, based solely on attention mechanisms, dispensing with recurrence and convolutions entirely. Experiments on two machine translation tasks show these models to be superior in quality while being more parallelizable and requiring significantly less time to train. Our model achieves 28.4 BLEU on the WMT 2014 English-to-German translation task, improving over the existing best results, including ensembles, by over 2 BLEU. On the WMT 2014 English-to-French translation task, our model establishes a new single-model state-of-the-art BLEU score of 41.8 after training for 3.5 days on eight GPUs, a small fraction of the training costs of the best models from the literature. We show that the Transformer generalizes well to other tasks by applying it successfully to English constituency parsing both with large and limited training data."#.to_string();
    paper.authors = vec![
        Author::new("Ashish Vaswani".to_string()),
        Author::new("Noam Shazeer".to_string()),
        Author::new("Niki Parmar".to_string()),
    ];
    paper
}

/// 簡単なプロンプトでAPI呼び出しをテスト
fn simple_test_messages() -> Vec<Message> {
    vec![
        Message::system("あなたは簡潔に回答するアシスタントです。"),
        Message::user("1 + 1 = ? 数字のみで回答してください。"),
    ]
}

// =============================================================================
// OpenAI 統合テスト
// =============================================================================

mod openai {
    use super::*;

    fn skip_if_no_api_key() -> Option<OpenAiProvider> {
        OpenAiProvider::from_env().ok()
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_openai_simple_completion() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("OPENAI_API_KEY が設定されていないためスキップ");
            return;
        };

        let config = LlmConfig::new()
            .with_model("gpt-4o-mini")
            .with_max_tokens(100);

        let result = provider.complete(simple_test_messages(), &config).await;

        assert!(result.is_ok(), "OpenAI APIエラー: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.is_empty(), "空のレスポンス");
        println!("OpenAI Response: {}", response);
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_openai_paper_analysis() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("OPENAI_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("gpt-4o-mini");

        let paper = create_sample_paper();
        let result = analyzer.analyze(&paper).await;

        assert!(result.is_ok(), "論文分析エラー: {:?}", result.err());

        let analysis = result.unwrap();
        assert!(!analysis.summary.is_empty(), "サマリーが空");
        assert!(!analysis.methodology.is_empty(), "方法論が空");
        assert!(!analysis.key_contributions.is_empty(), "貢献が空");
        assert_eq!(analysis.provider, "openai");

        println!("=== OpenAI 論文分析結果 ===");
        println!("サマリー: {}...", truncate_str(&analysis.summary, 200));
        println!("貢献: {:?}", analysis.key_contributions);
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_openai_generate_summary() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("OPENAI_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("gpt-4o-mini");

        let paper = create_sample_paper();
        let result = analyzer.generate_summary(&paper).await;

        assert!(result.is_ok(), "サマリー生成エラー: {:?}", result.err());

        let summary = result.unwrap();
        assert!(!summary.is_empty(), "サマリーが空");
        println!("=== OpenAI サマリー ===");
        println!("{}", truncate_str(&summary, 500));
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_openai_translate_to_japanese() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("OPENAI_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("gpt-4o-mini");

        let english_text = "The Transformer model revolutionized natural language processing.";
        let result = analyzer.translate_to_japanese(english_text).await;

        assert!(result.is_ok(), "翻訳エラー: {:?}", result.err());

        let translation = result.unwrap();
        assert!(!translation.is_empty(), "翻訳が空");
        // 日本語の文字が含まれているか確認
        assert!(
            translation
                .chars()
                .any(|c| c >= '\u{3040}' && c <= '\u{9FFF}'),
            "日本語文字が含まれていない: {}",
            translation
        );
        println!("=== OpenAI 翻訳 ===");
        println!("原文: {}", english_text);
        println!("翻訳: {}", translation);
    }
}

// =============================================================================
// Anthropic 統合テスト
// =============================================================================

mod anthropic {
    use super::*;

    fn skip_if_no_api_key() -> Option<AnthropicProvider> {
        AnthropicProvider::from_env().ok()
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_anthropic_simple_completion() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("ANTHROPIC_API_KEY が設定されていないためスキップ");
            return;
        };

        let config = LlmConfig::new()
            .with_model("claude-3-5-haiku-20241022")
            .with_max_tokens(100);

        let result = provider.complete(simple_test_messages(), &config).await;

        assert!(result.is_ok(), "Anthropic APIエラー: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.is_empty(), "空のレスポンス");
        println!("Anthropic Response: {}", response);
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_anthropic_paper_analysis() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("ANTHROPIC_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("claude-3-5-haiku-20241022");

        let paper = create_sample_paper();
        let result = analyzer.analyze(&paper).await;

        assert!(result.is_ok(), "論文分析エラー: {:?}", result.err());

        let analysis = result.unwrap();
        assert!(!analysis.summary.is_empty(), "サマリーが空");
        assert!(!analysis.methodology.is_empty(), "方法論が空");
        assert!(!analysis.key_contributions.is_empty(), "貢献が空");
        assert_eq!(analysis.provider, "anthropic");

        println!("=== Anthropic 論文分析結果 ===");
        println!("サマリー: {}...", truncate_str(&analysis.summary, 200));
        println!("貢献: {:?}", analysis.key_contributions);
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_anthropic_generate_summary() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("ANTHROPIC_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("claude-3-5-haiku-20241022");

        let paper = create_sample_paper();
        let result = analyzer.generate_summary(&paper).await;

        assert!(result.is_ok(), "サマリー生成エラー: {:?}", result.err());

        let summary = result.unwrap();
        assert!(!summary.is_empty(), "サマリーが空");
        println!("=== Anthropic サマリー ===");
        println!("{}", truncate_str(&summary, 500));
    }

    #[tokio::test]
    #[ignore = "APIキーが必要"]
    async fn test_anthropic_translate_to_japanese() {
        let Some(provider) = skip_if_no_api_key() else {
            eprintln!("ANTHROPIC_API_KEY が設定されていないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider).with_model("claude-3-5-haiku-20241022");

        let english_text = "The Transformer model revolutionized natural language processing.";
        let result = analyzer.translate_to_japanese(english_text).await;

        assert!(result.is_ok(), "翻訳エラー: {:?}", result.err());

        let translation = result.unwrap();
        assert!(!translation.is_empty(), "翻訳が空");
        assert!(
            translation
                .chars()
                .any(|c| c >= '\u{3040}' && c <= '\u{9FFF}'),
            "日本語文字が含まれていない: {}",
            translation
        );
        println!("=== Anthropic 翻訳 ===");
        println!("原文: {}", english_text);
        println!("翻訳: {}", translation);
    }
}

// =============================================================================
// Ollama 統合テスト
// =============================================================================

mod ollama {
    use super::*;

    async fn skip_if_ollama_unavailable() -> Option<OllamaProvider> {
        let provider = OllamaProvider::from_env().ok()?;

        // Ollamaサーバーへの接続テスト
        let client = reqwest::Client::new();
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let result = client.get(format!("{}/api/tags", base_url)).send().await;

        if result.is_err() {
            return None;
        }

        Some(provider)
    }

    #[tokio::test]
    #[ignore = "Ollamaサーバーが必要"]
    async fn test_ollama_simple_completion() {
        let Some(provider) = skip_if_ollama_unavailable().await else {
            eprintln!("Ollamaサーバーに接続できないためスキップ");
            return;
        };

        let config = LlmConfig::new().with_max_tokens(100);

        let result = provider.complete(simple_test_messages(), &config).await;

        assert!(result.is_ok(), "Ollama APIエラー: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.is_empty(), "空のレスポンス");
        println!("Ollama Response: {}", response);
    }

    #[tokio::test]
    #[ignore = "Ollamaサーバーが必要"]
    async fn test_ollama_paper_analysis() {
        let Some(provider) = skip_if_ollama_unavailable().await else {
            eprintln!("Ollamaサーバーに接続できないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider);

        let paper = create_sample_paper();
        let result = analyzer.analyze(&paper).await;

        assert!(result.is_ok(), "論文分析エラー: {:?}", result.err());

        let analysis = result.unwrap();
        assert!(!analysis.summary.is_empty(), "サマリーが空");
        assert_eq!(analysis.provider, "ollama");

        println!("=== Ollama 論文分析結果 ===");
        println!("サマリー: {}...", truncate_str(&analysis.summary, 200));
    }

    #[tokio::test]
    #[ignore = "Ollamaサーバーが必要"]
    async fn test_ollama_generate_summary() {
        let Some(provider) = skip_if_ollama_unavailable().await else {
            eprintln!("Ollamaサーバーに接続できないためスキップ");
            return;
        };

        let analyzer = PaperAnalyzer::new(provider);

        let paper = create_sample_paper();
        let result = analyzer.generate_summary(&paper).await;

        assert!(result.is_ok(), "サマリー生成エラー: {:?}", result.err());

        let summary = result.unwrap();
        assert!(!summary.is_empty(), "サマリーが空");
        println!("=== Ollama サマリー ===");
        println!("{}", truncate_str(&summary, 500));
    }
}

// =============================================================================
// プロバイダー比較テスト
// =============================================================================

#[tokio::test]
#[ignore = "全APIキーが必要"]
async fn test_compare_providers() {
    let paper = create_sample_paper();
    let mut results = Vec::new();

    // OpenAI
    if let Ok(provider) = OpenAiProvider::from_env() {
        let analyzer = PaperAnalyzer::new(provider).with_model("gpt-4o-mini");
        if let Ok(analysis) = analyzer.analyze(&paper).await {
            results.push(("OpenAI", analysis));
        }
    }

    // Anthropic
    if let Ok(provider) = AnthropicProvider::from_env() {
        let analyzer = PaperAnalyzer::new(provider).with_model("claude-3-5-haiku-20241022");
        if let Ok(analysis) = analyzer.analyze(&paper).await {
            results.push(("Anthropic", analysis));
        }
    }

    println!("\n=== プロバイダー比較結果 ===\n");
    for (name, analysis) in &results {
        println!("--- {} ---", name);
        println!("モデル: {}", analysis.model);
        println!("貢献数: {}", analysis.key_contributions.len());
        println!("タスク数: {}", analysis.tasks.len());
        println!(
            "サマリー冒頭: {}...\n",
            truncate_str(&analysis.summary, 150)
        );
    }

    assert!(
        !results.is_empty(),
        "どのプロバイダーも使用できませんでした"
    );
}
