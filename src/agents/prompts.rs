//! 論文分析用プロンプトテンプレート

/// LLMベースの論文分析用プロンプトテンプレート
pub struct PromptTemplates;

impl PromptTemplates {
    /// 論文分析用システムプロンプト
    pub fn system_prompt() -> &'static str {
        r#"あなたは複数の科学分野に深い知識を持つ学術論文分析の専門家です。研究論文を分析し、構造化された情報を抽出することがあなたの役割です。

ガイドライン:
- 簡潔かつ正確に記述してください
- 主要な貢献と新規性のある点に焦点を当ててください
- 専門用語を適切に使用してください
- 分析において客観性を保ってください
- アブストラクトに情報がない場合は「アブストラクトに記載なし」と示してください"#
    }

    /// 日本語翻訳用システムプロンプト
    pub fn japanese_translation_system() -> &'static str {
        r#"あなたは英語の学術論文を日本語に翻訳する専門の翻訳者です。専門用語の正確性と学術的なトーンを維持してください。"#
    }

    /// 論文サマリー生成用プロンプト
    pub fn summary_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"この学術論文を分析し、簡潔なサマリーを作成してください（2〜3段落）。

タイトル: {title}

アブストラクト: {abstract_text}

以下の点に焦点を当ててください:
1. 取り組んでいる主要な問題または研究課題
2. 提案されている解決策、手法、またはアプローチ
3. 主要な発見と貢献

論文の本質を捉えた、明確で構造化されたサマリーを提供してください。"#
        )
    }

    /// 方法論抽出用プロンプト
    pub fn methodology_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"この論文から方法論を抽出し、説明してください。

タイトル: {title}

アブストラクト: {abstract_text}

以下を記述してください:
1. 研究アプローチ（実験的、理論的、経験的など）
2. 使用されている主要な手法と技術
3. 評価方法論と指標
4. 方法論における新規性のある貢献

アブストラクトに方法論の詳細が限られている場合は、推測できる内容を記述してください。"#
        )
    }

    /// 完全論文分析用プロンプト（JSON出力）
    pub fn full_analysis_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"この学術論文を包括的に分析し、構造化された分析結果を提供してください。

タイトル: {title}

アブストラクト: {abstract_text}

以下の構造のJSONオブジェクトとして分析結果を提供してください:
{{
    "summary": "論文の2〜3段落のサマリー",
    "background_and_purpose": "研究の背景、動機、目的",
    "methodology": "技術的アプローチ、使用された手法と技術",
    "datasets": [
        {{
            "name": "データセット名（例: ImageNet, COCO, SQuAD）",
            "url": "データセットにアクセスできるURL（不明な場合は空文字）",
            "paper_title": "データセットを提案した元論文のタイトル（不明な場合は空文字）",
            "paper_url": "元論文のURL（不明な場合は空文字）",
            "paper_authors": "元論文の著者（不明な場合は空文字）",
            "description": "データセットの簡単な説明（不明な場合は空文字）",
            "domain": "分野（例: Computer Vision, NLP, Speech）",
            "size": "サイズ情報（例: 1.2M images, 100K samples、不明な場合は空文字）"
        }}
    ],
    "results": "主要な発見と実験結果",
    "advantages_limitations_and_future_work": "長所、短所、今後の方向性",
    "key_contributions": ["貢献1", "貢献2", ...],
    "tasks": ["研究分野1", "研究分野2", ...]
}}

datasetsは論文で使用されているすべてのデータセットのリストです。データセットが使用されていない場合や記載がない場合は空の配列[]を返してください。
すべてのフィールドを埋めてください。アブストラクトに情報がない場合は、合理的な推測を行うか「記載なし」と示してください。"#
        )
    }

    /// テキスト翻訳用プロンプト
    pub fn translation_prompt(text: &str, target_lang: &str) -> String {
        format!(
            r#"以下の学術テキストを{target_lang}に翻訳してください。

要件:
- 専門用語の正確性を維持
- 学術的なトーンとスタイルを保持
- 同じ段落構造を維持
- 適切な学術的表現を使用

翻訳するテキスト:
{text}

翻訳のみを提供し、説明は不要です。"#
        )
    }

    /// 主要貢献抽出用プロンプト
    pub fn key_contributions_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"この論文の主要な貢献を特定してください。

タイトル: {title}

アブストラクト: {abstract_text}

3〜5個の主要な貢献を箇条書きでリストアップしてください。各貢献は:
- 具体的で明確
- 明瞭に記述
- 先行研究からの新規性や進歩に焦点

文字列のJSON配列として出力してください:
["貢献1", "貢献2", ...]"#
        )
    }

    /// 研究タスク・分野識別用プロンプト
    pub fn research_tasks_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"この論文が取り組んでいる研究分野とタスクを特定してください。

タイトル: {title}

アブストラクト: {abstract_text}

関連する研究分野と具体的なタスクをリストアップしてください（例:「自然言語処理」「機械翻訳」「画像分類」）。

文字列のJSON配列として出力してください:
["分野/タスク1", "分野/タスク2", ...]"#
        )
    }

    /// キーワード・トピック抽出用プロンプト
    pub fn keyword_extraction_prompt(title: &str, abstract_text: &str) -> String {
        format!(
            r#"以下の学術論文からキーワード、トピック、技術用語を抽出してください。

タイトル: {title}

アブストラクト: {abstract_text}

以下の構造のJSONオブジェクトとして出力してください:
{{
    "keywords": ["主要キーワード1", "主要キーワード2", ...],
    "topics": ["研究トピック1", "研究トピック2", ...],
    "technical_terms": [
        {{"term": "技術用語1", "definition": "簡潔な定義"}},
        {{"term": "技術用語2", "definition": "簡潔な定義"}}
    ],
    "methods": ["手法1", "手法2", ...],
    "datasets": ["データセット1", "データセット2", ...]
}}

ガイドライン:
- keywords: 論文の主要な検索キーワード（5〜10個）
- topics: 研究分野・トピック（3〜5個）
- technical_terms: 重要な技術用語と定義（5個程度）
- methods: 使用されている手法・技術（該当するものすべて）
- datasets: 言及されているデータセット（該当するものすべて、なければ空配列）"#
        )
    }

    /// 研究コンテキスト生成用プロンプト
    pub fn research_context_prompt(
        title: &str,
        abstract_text: &str,
        keywords: &[String],
    ) -> String {
        let keywords_str = keywords.join(", ");
        format!(
            r#"以下の学術論文の研究分野における位置づけを分析してください。

タイトル: {title}

アブストラクト: {abstract_text}

キーワード: {keywords_str}

以下の構造のJSONオブジェクトとして出力してください:
{{
    "primary_field": "主要研究分野",
    "sub_fields": ["サブ分野1", "サブ分野2", ...],
    "research_type": "研究タイプ（以下から選択: empirical, theoretical, survey, methodology, application）",
    "positioning": "この研究の分野における位置づけの説明（2〜3文）",
    "related_directions": ["関連研究方向1", "関連研究方向2", ...]
}}

ガイドライン:
- primary_field: 最も関連性の高い主要研究分野
- sub_fields: より具体的なサブ分野（2〜4個）
- research_type: empirical（実験的）, theoretical（理論的）, survey（調査）, methodology（方法論提案）, application（応用）のいずれか
- positioning: この研究が分野にどのように貢献し、どのような問題を解決しようとしているか
- related_directions: この研究から発展しうる関連研究方向（3〜5個）"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let prompt = PromptTemplates::system_prompt();
        assert!(prompt.contains("学術論文分析の専門家"));
    }

    #[test]
    fn test_summary_prompt() {
        let prompt = PromptTemplates::summary_prompt("Test Title", "Test abstract");
        assert!(prompt.contains("Test Title"));
        assert!(prompt.contains("Test abstract"));
    }

    #[test]
    fn test_full_analysis_prompt_contains_json() {
        let prompt = PromptTemplates::full_analysis_prompt("Title", "Abstract");
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("summary"));
        assert!(prompt.contains("methodology"));
    }
}
