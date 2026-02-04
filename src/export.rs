//! Export data structures for comprehensive paper data output
//!
//! This module provides structures for exporting academic paper data
//! in a format optimized for LLM/AI agent consumption.

use crate::models::AcademicPaper;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current schema version for export format
pub const EXPORT_SCHEMA_VERSION: &str = "1.0.0";

/// XML Schema (XSD) for exported paper data
pub const EXPORTED_PAPER_XSD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           elementFormDefault="qualified">

  <xs:annotation>
    <xs:documentation xml:lang="ja">
      学術論文エクスポートデータのXMLスキーマ定義
      AI/LLMによる論文解析結果を含む包括的な論文データ形式
    </xs:documentation>
  </xs:annotation>

  <!-- ルート要素 -->
  <xs:element name="exported-paper">
    <xs:annotation>
      <xs:documentation xml:lang="ja">エクスポートされた論文データのルート要素</xs:documentation>
    </xs:annotation>
    <xs:complexType>
      <xs:sequence>
        <xs:element name="export-metadata" type="ExportMetadataType">
          <xs:annotation>
            <xs:documentation xml:lang="ja">エクスポート処理のメタデータ</xs:documentation>
          </xs:annotation>
        </xs:element>
        <xs:element name="paper" type="PaperType">
          <xs:annotation>
            <xs:documentation xml:lang="ja">論文の本体データ</xs:documentation>
          </xs:annotation>
        </xs:element>
        <xs:element name="citations" type="CitationsType" minOccurs="0">
          <xs:annotation>
            <xs:documentation xml:lang="ja">この論文を引用している論文のリスト（オプション）</xs:documentation>
          </xs:annotation>
        </xs:element>
        <xs:element name="references" type="ReferencesType" minOccurs="0">
          <xs:annotation>
            <xs:documentation xml:lang="ja">この論文が参照している論文のリスト（オプション）</xs:documentation>
          </xs:annotation>
        </xs:element>
        <xs:element name="keywords-data" type="KeywordsDataType" minOccurs="0">
          <xs:annotation>
            <xs:documentation xml:lang="ja">LLMで抽出されたキーワードとトピック（オプション）</xs:documentation>
          </xs:annotation>
        </xs:element>
        <xs:element name="research-context" type="ResearchContextType" minOccurs="0">
          <xs:annotation>
            <xs:documentation xml:lang="ja">研究分野における位置づけ情報（オプション）</xs:documentation>
          </xs:annotation>
        </xs:element>
      </xs:sequence>
      <xs:attribute name="schema-version" type="xs:string" use="required">
        <xs:annotation>
          <xs:documentation xml:lang="ja">スキーマバージョン（互換性確認用）</xs:documentation>
        </xs:annotation>
      </xs:attribute>
    </xs:complexType>
  </xs:element>

  <!-- エクスポートメタデータ型 -->
  <xs:complexType name="ExportMetadataType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">エクスポート処理に関するメタデータ</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="exported-at" type="xs:dateTime">
        <xs:annotation>
          <xs:documentation xml:lang="ja">エクスポート実行日時（ISO 8601形式）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="tool-version" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">エクスポートツールのバージョン</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="options" type="ExportOptionsType">
        <xs:annotation>
          <xs:documentation xml:lang="ja">エクスポート時に使用されたオプション</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="warnings" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">エクスポート処理中に発生した警告メッセージ</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="warning" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別の警告メッセージ</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- エクスポートオプション型 -->
  <xs:complexType name="ExportOptionsType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">エクスポート時のオプション設定（再現性のため記録）</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="analyzed" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">LLM分析が実行されたかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="text-extracted" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">PDFからテキスト抽出が実行されたかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="citations-included" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">引用論文が含まれているかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="references-included" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">参照論文が含まれているかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="keywords-extracted" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">キーワード抽出が実行されたかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="max-citations" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">取得した引用/参照論文の最大数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="llm-provider" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">使用されたLLMプロバイダー（openai, anthropic, ollama）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="llm-model" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">使用されたLLMモデル名</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 論文型 -->
  <xs:complexType name="PaperType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">学術論文の完全なデータ構造</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="identifiers" type="IdentifiersType">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文の各種識別子</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="metadata" type="MetadataType">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文のメタデータ（タイトル、著者、アブストラクト等）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="metrics" type="MetricsType">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文の計量情報（引用数等）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="analysis" type="AnalysisType" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">LLMによる論文分析結果（オプション）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="extracted-text" type="ExtractedTextType" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">PDFから抽出された本文テキスト（オプション）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="created-at" type="xs:dateTime">
        <xs:annotation>
          <xs:documentation xml:lang="ja">レコード作成日時</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="updated-at" type="xs:dateTime">
        <xs:annotation>
          <xs:documentation xml:lang="ja">レコード最終更新日時</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 識別子型 -->
  <xs:complexType name="IdentifiersType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">論文を一意に識別するための各種ID</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="ss-id" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">Semantic Scholar論文ID</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="arxiv-id" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">arXiv論文ID（例：2106.09685）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="doi" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">Digital Object Identifier</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- メタデータ型 -->
  <xs:complexType name="MetadataType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">論文の書誌情報</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="title" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文タイトル</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="authors">
        <xs:annotation>
          <xs:documentation xml:lang="ja">著者リスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="author" type="AuthorType" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別の著者情報</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="abstract" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文アブストラクト（要旨）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="abstract-ja" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">アブストラクトの日本語訳</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="url" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文のURL</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="journal" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">掲載ジャーナル名または会議名</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="primary-category" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">主要arXivカテゴリ（例：cs.CL, cs.AI）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="categories" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">全arXivカテゴリリスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="category" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のarXivカテゴリ</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="published-date" type="xs:date">
        <xs:annotation>
          <xs:documentation xml:lang="ja">出版日（YYYY-MM-DD形式）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="bibtex" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">BibTeX形式の引用情報</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 著者型 -->
  <xs:complexType name="AuthorType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">著者の詳細情報</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="name" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">著者名</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="ss-id" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">Semantic Scholar著者ID</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="h-index" type="xs:integer" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">h-index（研究者の生産性と影響力の指標）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="affiliations" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">所属機関リスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="affiliation" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">所属機関名</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="paper-count" type="xs:integer" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">著者の総論文数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="citation-count" type="xs:integer" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">著者の総被引用数</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 計量情報型 -->
  <xs:complexType name="MetricsType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">論文の計量的指標</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="citations-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">被引用数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="references-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">参照論文数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="influential-citation-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">影響力のある引用の数（Semantic Scholar指標）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="is-open-access" type="xs:boolean">
        <xs:annotation>
          <xs:documentation xml:lang="ja">オープンアクセスかどうか</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="open-access-pdf-url" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">オープンアクセスPDFのURL</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- LLM分析型 -->
  <xs:complexType name="AnalysisType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">LLMによる論文分析結果</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="summary" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文の要約（2-3段落）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="summary-ja" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">要約の日本語訳</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="background-and-purpose" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">研究背景と目的</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="methodology" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">手法と技術的アプローチ</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="dataset" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">使用されたデータセット</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="results" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">主要な結果と発見</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="advantages-limitations-and-future-work" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">利点、制限事項、および将来の研究方向</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="key-contributions" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">主要な貢献点のリスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="contribution" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別の貢献点</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="tasks" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">研究タスクカテゴリ（例：NLP, Computer Vision）</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="task" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のタスクカテゴリ</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="analyzed-at" type="xs:dateTime">
        <xs:annotation>
          <xs:documentation xml:lang="ja">分析実行日時</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="provider" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">分析に使用したLLMプロバイダー</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="model" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">分析に使用したLLMモデル</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 抽出テキスト型 -->
  <xs:complexType name="ExtractedTextType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">PDFから抽出されたテキストデータ</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="extracted-at" type="xs:dateTime">
        <xs:annotation>
          <xs:documentation xml:lang="ja">テキスト抽出実行日時</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="source-url" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">抽出元PDFのURL</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="sections">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文のセクションリスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="section" type="SectionType" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のセクション</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- セクション型 -->
  <xs:complexType name="SectionType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">論文の個別セクション（Abstract, Introduction等）</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="title" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">セクションタイトル</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="content" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">セクションの本文内容</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
    <xs:attribute name="index" type="xs:integer" use="required">
      <xs:annotation>
        <xs:documentation xml:lang="ja">セクション番号（論文内での順序）</xs:documentation>
      </xs:annotation>
    </xs:attribute>
    <xs:attribute name="importance" type="ImportanceType" use="required">
      <xs:annotation>
        <xs:documentation xml:lang="ja">セクションの重要度レベル</xs:documentation>
      </xs:annotation>
    </xs:attribute>
  </xs:complexType>

  <!-- 重要度型 -->
  <xs:simpleType name="ImportanceType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">
        セクションの重要度レベル:
        - critical: 最重要（Abstract, Method, Experiments）
        - high: 高重要度（Introduction, Results, Discussion）
        - medium: 中重要度（Related Work, Background, Conclusion）
        - reference: 参照用（Appendix, References, Acknowledgements）
      </xs:documentation>
    </xs:annotation>
    <xs:restriction base="xs:string">
      <xs:enumeration value="critical"/>
      <xs:enumeration value="high"/>
      <xs:enumeration value="medium"/>
      <xs:enumeration value="reference"/>
    </xs:restriction>
  </xs:simpleType>

  <!-- 引用論文型 -->
  <xs:complexType name="CitationsType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">この論文を引用している論文のデータ</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="total-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">APIから取得した総引用数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="fetched-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">実際に取得した引用論文数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="papers">
        <xs:annotation>
          <xs:documentation xml:lang="ja">引用論文のリスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="paper-summary" type="PaperSummaryType" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">引用論文の要約情報</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 参照論文型 -->
  <xs:complexType name="ReferencesType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">この論文が参照している論文のデータ</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="total-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">APIから取得した総参照数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="fetched-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">実際に取得した参照論文数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="papers">
        <xs:annotation>
          <xs:documentation xml:lang="ja">参照論文のリスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="paper-summary" type="PaperSummaryType" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">参照論文の要約情報</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 論文要約型 -->
  <xs:complexType name="PaperSummaryType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">引用/参照論文の簡略化された情報</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="ss-id" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">Semantic Scholar論文ID</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="arxiv-id" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">arXiv論文ID</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="doi" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">Digital Object Identifier</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="title" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文タイトル</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="authors">
        <xs:annotation>
          <xs:documentation xml:lang="ja">著者名リスト（簡略版）</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="author" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">著者名</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="year" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">出版年</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="venue" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">掲載誌/会議名</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="citation-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">被引用数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="influential-citation-count" type="xs:integer">
        <xs:annotation>
          <xs:documentation xml:lang="ja">影響力のある引用の数</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="abstract-snippet" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">アブストラクトの抜粋（長い場合は切り詰め）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="url" type="xs:string" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文URL</xs:documentation>
        </xs:annotation>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- キーワードデータ型 -->
  <xs:complexType name="KeywordsDataType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">LLMで抽出されたキーワードと関連情報</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="keywords" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">主要な研究キーワード</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="keyword" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のキーワード</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="topics" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">研究トピック/ドメイン</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="topic" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のトピック</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="methods" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文で言及された手法/技術</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="method" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別の手法名</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="datasets" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">論文で言及されたデータセット</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="dataset" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のデータセット名</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

  <!-- 研究コンテキスト型 -->
  <xs:complexType name="ResearchContextType">
    <xs:annotation>
      <xs:documentation xml:lang="ja">研究分野における論文の位置づけ情報</xs:documentation>
    </xs:annotation>
    <xs:sequence>
      <xs:element name="primary-field" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">主要研究分野</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="sub-fields" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">サブ研究分野リスト</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="sub-field" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別のサブ分野</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
      <xs:element name="research-type" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">研究タイプ（empirical, theoretical, survey等）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="positioning" type="xs:string">
        <xs:annotation>
          <xs:documentation xml:lang="ja">研究分野における位置づけの説明（LLM生成）</xs:documentation>
        </xs:annotation>
      </xs:element>
      <xs:element name="related-directions" minOccurs="0">
        <xs:annotation>
          <xs:documentation xml:lang="ja">関連する研究方向</xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="direction" type="xs:string" maxOccurs="unbounded">
              <xs:annotation>
                <xs:documentation xml:lang="ja">個別の研究方向</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:sequence>
  </xs:complexType>

</xs:schema>
"#;

/// Get XML Schema content
pub fn get_xml_schema() -> &'static str {
    EXPORTED_PAPER_XSD
}

/// Comprehensive exported paper data for AI/LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedPaper {
    /// Schema version for compatibility checking
    pub schema_version: String,

    /// Export metadata
    pub export_metadata: ExportMetadata,

    /// Main paper data
    pub paper: AcademicPaper,

    /// Papers citing this paper (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationData>,

    /// Papers referenced by this paper (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<ReferenceData>,

    /// Extracted keywords and topics (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<KeywordsData>,

    /// Research positioning context (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub research_context: Option<ResearchContext>,
}

impl ExportedPaper {
    /// Create a new ExportedPaper with default metadata
    pub fn new(paper: AcademicPaper, options: ExportOptions) -> Self {
        Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            export_metadata: ExportMetadata {
                exported_at: Local::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                options,
                warnings: Vec::new(),
            },
            paper,
            citations: None,
            references: None,
            keywords: None,
            research_context: None,
        }
    }

    /// Add a warning message
    pub fn add_warning(&mut self, warning: String) {
        self.export_metadata.warnings.push(warning);
    }

    /// Convert to XML format with all paper information
    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<exported-paper schema-version=\"{}\">\n",
            escape_xml(&self.schema_version)
        ));

        // Export metadata
        xml.push_str("  <export-metadata>\n");
        xml.push_str(&format!(
            "    <exported-at>{}</exported-at>\n",
            self.export_metadata
                .exported_at
                .format("%Y-%m-%dT%H:%M:%S%z")
        ));
        xml.push_str(&format!(
            "    <tool-version>{}</tool-version>\n",
            escape_xml(&self.export_metadata.tool_version)
        ));
        xml.push_str("    <options>\n");
        xml.push_str(&format!(
            "      <analyzed>{}</analyzed>\n",
            self.export_metadata.options.analyzed
        ));
        xml.push_str(&format!(
            "      <text-extracted>{}</text-extracted>\n",
            self.export_metadata.options.text_extracted
        ));
        xml.push_str(&format!(
            "      <citations-included>{}</citations-included>\n",
            self.export_metadata.options.citations_included
        ));
        xml.push_str(&format!(
            "      <references-included>{}</references-included>\n",
            self.export_metadata.options.references_included
        ));
        xml.push_str(&format!(
            "      <keywords-extracted>{}</keywords-extracted>\n",
            self.export_metadata.options.keywords_extracted
        ));
        xml.push_str(&format!(
            "      <max-citations>{}</max-citations>\n",
            self.export_metadata.options.max_citations
        ));
        if let Some(ref provider) = self.export_metadata.options.llm_provider {
            xml.push_str(&format!(
                "      <llm-provider>{}</llm-provider>\n",
                escape_xml(provider)
            ));
        }
        if let Some(ref model) = self.export_metadata.options.llm_model {
            xml.push_str(&format!(
                "      <llm-model>{}</llm-model>\n",
                escape_xml(model)
            ));
        }
        xml.push_str("    </options>\n");
        if !self.export_metadata.warnings.is_empty() {
            xml.push_str("    <warnings>\n");
            for warning in &self.export_metadata.warnings {
                xml.push_str(&format!(
                    "      <warning>{}</warning>\n",
                    escape_xml(warning)
                ));
            }
            xml.push_str("    </warnings>\n");
        }
        xml.push_str("  </export-metadata>\n\n");

        // Paper data
        xml.push_str("  <paper>\n");
        xml.push_str(&self.paper_to_xml());
        xml.push_str("  </paper>\n");

        // Citations
        if let Some(ref citations) = self.citations {
            xml.push_str("\n  <citations>\n");
            xml.push_str(&format!(
                "    <total-count>{}</total-count>\n",
                citations.total_count
            ));
            xml.push_str(&format!(
                "    <fetched-count>{}</fetched-count>\n",
                citations.fetched_count
            ));
            xml.push_str("    <papers>\n");
            for paper in &citations.papers {
                xml.push_str(&self.paper_summary_to_xml(paper, 6));
            }
            xml.push_str("    </papers>\n");
            xml.push_str("  </citations>\n");
        }

        // References
        if let Some(ref references) = self.references {
            xml.push_str("\n  <references>\n");
            xml.push_str(&format!(
                "    <total-count>{}</total-count>\n",
                references.total_count
            ));
            xml.push_str(&format!(
                "    <fetched-count>{}</fetched-count>\n",
                references.fetched_count
            ));
            xml.push_str("    <papers>\n");
            for paper in &references.papers {
                xml.push_str(&self.paper_summary_to_xml(paper, 6));
            }
            xml.push_str("    </papers>\n");
            xml.push_str("  </references>\n");
        }

        // Keywords
        if let Some(ref keywords) = self.keywords {
            xml.push_str("\n  <keywords-data>\n");
            if !keywords.keywords.is_empty() {
                xml.push_str("    <keywords>\n");
                for kw in &keywords.keywords {
                    xml.push_str(&format!("      <keyword>{}</keyword>\n", escape_xml(kw)));
                }
                xml.push_str("    </keywords>\n");
            }
            if !keywords.topics.is_empty() {
                xml.push_str("    <topics>\n");
                for topic in &keywords.topics {
                    xml.push_str(&format!("      <topic>{}</topic>\n", escape_xml(topic)));
                }
                xml.push_str("    </topics>\n");
            }
            if !keywords.methods.is_empty() {
                xml.push_str("    <methods>\n");
                for method in &keywords.methods {
                    xml.push_str(&format!("      <method>{}</method>\n", escape_xml(method)));
                }
                xml.push_str("    </methods>\n");
            }
            if !keywords.datasets.is_empty() {
                xml.push_str("    <datasets>\n");
                for dataset in &keywords.datasets {
                    xml.push_str(&format!(
                        "      <dataset>{}</dataset>\n",
                        escape_xml(dataset)
                    ));
                }
                xml.push_str("    </datasets>\n");
            }
            xml.push_str("  </keywords-data>\n");
        }

        // Research context
        if let Some(ref context) = self.research_context {
            xml.push_str("\n  <research-context>\n");
            xml.push_str(&format!(
                "    <primary-field>{}</primary-field>\n",
                escape_xml(&context.primary_field)
            ));
            if !context.sub_fields.is_empty() {
                xml.push_str("    <sub-fields>\n");
                for field in &context.sub_fields {
                    xml.push_str(&format!(
                        "      <sub-field>{}</sub-field>\n",
                        escape_xml(field)
                    ));
                }
                xml.push_str("    </sub-fields>\n");
            }
            xml.push_str(&format!(
                "    <research-type>{}</research-type>\n",
                escape_xml(&context.research_type)
            ));
            xml.push_str(&format!(
                "    <positioning>{}</positioning>\n",
                escape_xml(&context.positioning)
            ));
            if !context.related_directions.is_empty() {
                xml.push_str("    <related-directions>\n");
                for dir in &context.related_directions {
                    xml.push_str(&format!(
                        "      <direction>{}</direction>\n",
                        escape_xml(dir)
                    ));
                }
                xml.push_str("    </related-directions>\n");
            }
            xml.push_str("  </research-context>\n");
        }

        xml.push_str("</exported-paper>");
        xml
    }

    /// Convert paper data to XML
    fn paper_to_xml(&self) -> String {
        let paper = &self.paper;
        let mut xml = String::new();

        // Identifiers
        xml.push_str("    <identifiers>\n");
        if !paper.ss_id.is_empty() {
            xml.push_str(&format!(
                "      <ss-id>{}</ss-id>\n",
                escape_xml(&paper.ss_id)
            ));
        }
        if !paper.arxiv_id.is_empty() {
            xml.push_str(&format!(
                "      <arxiv-id>{}</arxiv-id>\n",
                escape_xml(&paper.arxiv_id)
            ));
        }
        if !paper.doi.is_empty() {
            xml.push_str(&format!("      <doi>{}</doi>\n", escape_xml(&paper.doi)));
        }
        xml.push_str("    </identifiers>\n\n");

        // Metadata
        xml.push_str("    <metadata>\n");
        xml.push_str(&format!(
            "      <title>{}</title>\n",
            escape_xml(&paper.title)
        ));
        xml.push_str("      <authors>\n");
        for author in &paper.authors {
            xml.push_str("        <author>\n");
            xml.push_str(&format!(
                "          <name>{}</name>\n",
                escape_xml(&author.name)
            ));
            if !author.ss_id.is_empty() {
                xml.push_str(&format!(
                    "          <ss-id>{}</ss-id>\n",
                    escape_xml(&author.ss_id)
                ));
            }
            if author.h_index > 0 {
                xml.push_str(&format!(
                    "          <h-index>{}</h-index>\n",
                    author.h_index
                ));
            }
            if !author.affiliations.is_empty() {
                xml.push_str("          <affiliations>\n");
                for aff in &author.affiliations {
                    xml.push_str(&format!(
                        "            <affiliation>{}</affiliation>\n",
                        escape_xml(aff)
                    ));
                }
                xml.push_str("          </affiliations>\n");
            }
            if author.paper_count > 0 {
                xml.push_str(&format!(
                    "          <paper-count>{}</paper-count>\n",
                    author.paper_count
                ));
            }
            if author.citation_count > 0 {
                xml.push_str(&format!(
                    "          <citation-count>{}</citation-count>\n",
                    author.citation_count
                ));
            }
            xml.push_str("        </author>\n");
        }
        xml.push_str("      </authors>\n");
        xml.push_str(&format!(
            "      <abstract>{}</abstract>\n",
            escape_xml(&paper.abstract_text)
        ));
        if !paper.abstract_text_ja.is_empty() {
            xml.push_str(&format!(
                "      <abstract-ja>{}</abstract-ja>\n",
                escape_xml(&paper.abstract_text_ja)
            ));
        }
        xml.push_str(&format!("      <url>{}</url>\n", escape_xml(&paper.url)));
        if !paper.journal.is_empty() {
            xml.push_str(&format!(
                "      <journal>{}</journal>\n",
                escape_xml(&paper.journal)
            ));
        }
        if !paper.primary_category.is_empty() {
            xml.push_str(&format!(
                "      <primary-category>{}</primary-category>\n",
                escape_xml(&paper.primary_category)
            ));
        }
        if !paper.categories.is_empty() {
            xml.push_str("      <categories>\n");
            for cat in &paper.categories {
                xml.push_str(&format!(
                    "        <category>{}</category>\n",
                    escape_xml(cat)
                ));
            }
            xml.push_str("      </categories>\n");
        }
        xml.push_str(&format!(
            "      <published-date>{}</published-date>\n",
            paper.published_date.format("%Y-%m-%d")
        ));
        if !paper.bibtex.is_empty() {
            xml.push_str(&format!(
                "      <bibtex><![CDATA[{}]]></bibtex>\n",
                paper.bibtex
            ));
        }
        xml.push_str("    </metadata>\n\n");

        // Metrics
        xml.push_str("    <metrics>\n");
        xml.push_str(&format!(
            "      <citations-count>{}</citations-count>\n",
            paper.citations_count
        ));
        xml.push_str(&format!(
            "      <references-count>{}</references-count>\n",
            paper.references_count
        ));
        xml.push_str(&format!(
            "      <influential-citation-count>{}</influential-citation-count>\n",
            paper.influential_citation_count
        ));
        xml.push_str(&format!(
            "      <is-open-access>{}</is-open-access>\n",
            paper.is_open_access
        ));
        if let Some(ref url) = paper.open_access_pdf_url {
            xml.push_str(&format!(
                "      <open-access-pdf-url>{}</open-access-pdf-url>\n",
                escape_xml(url)
            ));
        }
        xml.push_str("    </metrics>\n");

        // Analysis
        if let Some(ref analysis) = paper.analysis {
            xml.push_str("\n    <analysis>\n");
            xml.push_str(&format!(
                "      <summary>{}</summary>\n",
                escape_xml(&analysis.summary)
            ));
            xml.push_str(&format!(
                "      <background-and-purpose>{}</background-and-purpose>\n",
                escape_xml(&analysis.background_and_purpose)
            ));
            xml.push_str(&format!(
                "      <methodology>{}</methodology>\n",
                escape_xml(&analysis.methodology)
            ));
            if !analysis.datasets.is_empty() {
                for ds in &analysis.datasets {
                    xml.push_str(&format!(
                        "      <dataset>{}</dataset>\n",
                        escape_xml(&ds.name)
                    ));
                }
            }
            xml.push_str(&format!(
                "      <results>{}</results>\n",
                escape_xml(&analysis.results)
            ));
            xml.push_str(&format!(
                "      <advantages-limitations-and-future-work>{}</advantages-limitations-and-future-work>\n",
                escape_xml(&analysis.advantages_limitations_and_future_work)
            ));
            if !analysis.key_contributions.is_empty() {
                xml.push_str("      <key-contributions>\n");
                for contrib in &analysis.key_contributions {
                    xml.push_str(&format!(
                        "        <contribution>{}</contribution>\n",
                        escape_xml(contrib)
                    ));
                }
                xml.push_str("      </key-contributions>\n");
            }
            if !analysis.tasks.is_empty() {
                xml.push_str("      <tasks>\n");
                for task in &analysis.tasks {
                    xml.push_str(&format!("        <task>{}</task>\n", escape_xml(task)));
                }
                xml.push_str("      </tasks>\n");
            }
            xml.push_str(&format!(
                "      <analyzed-at>{}</analyzed-at>\n",
                analysis.analyzed_at.format("%Y-%m-%dT%H:%M:%S%z")
            ));
            xml.push_str(&format!(
                "      <provider>{}</provider>\n",
                escape_xml(&analysis.provider)
            ));
            xml.push_str(&format!(
                "      <model>{}</model>\n",
                escape_xml(&analysis.model)
            ));
            xml.push_str("    </analysis>\n");
        }

        // Extracted text
        if let Some(ref text) = paper.extracted_text {
            xml.push_str("\n    <extracted-text>\n");
            xml.push_str(&format!(
                "      <extracted-at>{}</extracted-at>\n",
                text.extracted_at.format("%Y-%m-%dT%H:%M:%S%z")
            ));
            xml.push_str(&format!(
                "      <source-url>{}</source-url>\n",
                escape_xml(&text.source_url)
            ));
            xml.push_str("      <sections>\n");
            for section in &text.sections {
                xml.push_str(&format!(
                    "        <section index=\"{}\" importance=\"{}\">\n",
                    section.index,
                    section.importance.as_str()
                ));
                xml.push_str(&format!(
                    "          <title>{}</title>\n",
                    escape_xml(&section.title)
                ));
                xml.push_str(&format!(
                    "          <content>{}</content>\n",
                    escape_xml(&section.content)
                ));
                xml.push_str("        </section>\n");
            }
            xml.push_str("      </sections>\n");
            xml.push_str("    </extracted-text>\n");
        }

        // Timestamps
        xml.push_str(&format!(
            "\n    <created-at>{}</created-at>\n",
            paper.created_at.format("%Y-%m-%dT%H:%M:%S%z")
        ));
        xml.push_str(&format!(
            "    <updated-at>{}</updated-at>\n",
            paper.updated_at.format("%Y-%m-%dT%H:%M:%S%z")
        ));

        xml
    }

    /// Convert paper summary to XML with specified indentation
    fn paper_summary_to_xml(&self, paper: &PaperSummary, indent: usize) -> String {
        let ind = " ".repeat(indent);
        let mut xml = format!("{}<paper-summary>\n", ind);
        if !paper.ss_id.is_empty() {
            xml.push_str(&format!(
                "{}  <ss-id>{}</ss-id>\n",
                ind,
                escape_xml(&paper.ss_id)
            ));
        }
        if !paper.arxiv_id.is_empty() {
            xml.push_str(&format!(
                "{}  <arxiv-id>{}</arxiv-id>\n",
                ind,
                escape_xml(&paper.arxiv_id)
            ));
        }
        if !paper.doi.is_empty() {
            xml.push_str(&format!("{}  <doi>{}</doi>\n", ind, escape_xml(&paper.doi)));
        }
        xml.push_str(&format!(
            "{}  <title>{}</title>\n",
            ind,
            escape_xml(&paper.title)
        ));
        xml.push_str(&format!("{}  <authors>\n", ind));
        for author in &paper.authors {
            xml.push_str(&format!(
                "{}    <author>{}</author>\n",
                ind,
                escape_xml(author)
            ));
        }
        xml.push_str(&format!("{}  </authors>\n", ind));
        xml.push_str(&format!("{}  <year>{}</year>\n", ind, paper.year));
        if !paper.venue.is_empty() {
            xml.push_str(&format!(
                "{}  <venue>{}</venue>\n",
                ind,
                escape_xml(&paper.venue)
            ));
        }
        xml.push_str(&format!(
            "{}  <citation-count>{}</citation-count>\n",
            ind, paper.citation_count
        ));
        xml.push_str(&format!(
            "{}  <influential-citation-count>{}</influential-citation-count>\n",
            ind, paper.influential_citation_count
        ));
        if !paper.abstract_snippet.is_empty() {
            xml.push_str(&format!(
                "{}  <abstract-snippet>{}</abstract-snippet>\n",
                ind,
                escape_xml(&paper.abstract_snippet)
            ));
        }
        if !paper.url.is_empty() {
            xml.push_str(&format!("{}  <url>{}</url>\n", ind, escape_xml(&paper.url)));
        }
        xml.push_str(&format!("{}</paper-summary>\n", ind));
        xml
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Export operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// When the export was performed
    pub exported_at: DateTime<Local>,

    /// Tool version
    pub tool_version: String,

    /// Options used for this export
    pub options: ExportOptions,

    /// Any warnings or notes about the export
    pub warnings: Vec<String>,
}

/// Options used during export (for reproducibility)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Whether LLM analysis was performed
    pub analyzed: bool,

    /// Whether PDF text was extracted
    pub text_extracted: bool,

    /// Whether citations were included
    pub citations_included: bool,

    /// Whether references were included
    pub references_included: bool,

    /// Whether keywords were extracted
    pub keywords_extracted: bool,

    /// Maximum number of citations/references fetched
    pub max_citations: usize,

    /// LLM provider used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,

    /// LLM model used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
}

/// Citation network data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationData {
    /// Total citation count from API
    pub total_count: i32,

    /// Number of citations fetched
    pub fetched_count: usize,

    /// Citing papers (simplified)
    pub papers: Vec<PaperSummary>,

    /// Citation statistics
    pub statistics: CitationStatistics,
}

/// Reference network data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceData {
    /// Total reference count from API
    pub total_count: i32,

    /// Number of references fetched
    pub fetched_count: usize,

    /// Referenced papers (simplified)
    pub papers: Vec<PaperSummary>,

    /// Reference statistics
    pub statistics: ReferenceStatistics,
}

/// Simplified paper representation for citations/references
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperSummary {
    /// Semantic Scholar paper ID
    pub ss_id: String,

    /// arXiv paper ID
    pub arxiv_id: String,

    /// Digital Object Identifier
    pub doi: String,

    /// Paper title
    pub title: String,

    /// Author names (just names for brevity)
    pub authors: Vec<String>,

    /// Publication year
    pub year: i32,

    /// Journal or venue name
    pub venue: String,

    /// Citation count
    pub citation_count: i32,

    /// Influential citation count
    pub influential_citation_count: i32,

    /// Brief abstract (truncated if too long)
    pub abstract_snippet: String,

    /// Paper URL
    pub url: String,
}

impl PaperSummary {
    /// Create a PaperSummary from AcademicPaper
    pub fn from_academic_paper(paper: &AcademicPaper) -> Self {
        let abstract_snippet = if paper.abstract_text.len() > 500 {
            format!("{}...", &paper.abstract_text[..500])
        } else {
            paper.abstract_text.clone()
        };

        Self {
            ss_id: paper.ss_id.clone(),
            arxiv_id: paper.arxiv_id.clone(),
            doi: paper.doi.clone(),
            title: paper.title.clone(),
            authors: paper.authors.iter().map(|a| a.name.clone()).collect(),
            year: paper
                .published_date
                .format("%Y")
                .to_string()
                .parse()
                .unwrap_or(0),
            venue: paper.journal.clone(),
            citation_count: paper.citations_count,
            influential_citation_count: paper.influential_citation_count,
            abstract_snippet,
            url: paper.url.clone(),
        }
    }
}

/// Statistics about citations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationStatistics {
    /// Year distribution of citations
    pub by_year: HashMap<i32, usize>,

    /// Top venues citing this paper
    pub top_venues: Vec<(String, usize)>,

    /// Average citations of citing papers
    pub avg_citation_count: f64,

    /// Most influential citing papers (top 5 by citation count)
    pub most_influential: Vec<String>,
}

impl CitationStatistics {
    /// Calculate statistics from a list of papers
    pub fn from_papers(papers: &[PaperSummary]) -> Self {
        let mut by_year: HashMap<i32, usize> = HashMap::new();
        let mut venues: HashMap<String, usize> = HashMap::new();
        let mut total_citations = 0i64;

        for paper in papers {
            if paper.year > 0 {
                *by_year.entry(paper.year).or_insert(0) += 1;
            }
            if !paper.venue.is_empty() {
                *venues.entry(paper.venue.clone()).or_insert(0) += 1;
            }
            total_citations += paper.citation_count as i64;
        }

        // Sort venues by count and take top 10
        let mut venue_vec: Vec<_> = venues.into_iter().collect();
        venue_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let top_venues: Vec<_> = venue_vec.into_iter().take(10).collect();

        // Get most influential papers (top 5 by citation count)
        let mut sorted_papers: Vec<_> = papers.iter().collect();
        sorted_papers.sort_by(|a, b| b.citation_count.cmp(&a.citation_count));
        let most_influential: Vec<_> = sorted_papers
            .iter()
            .take(5)
            .map(|p| p.title.clone())
            .collect();

        let avg_citation_count = if papers.is_empty() {
            0.0
        } else {
            total_citations as f64 / papers.len() as f64
        };

        Self {
            by_year,
            top_venues,
            avg_citation_count,
            most_influential,
        }
    }
}

/// Statistics about references
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReferenceStatistics {
    /// Year distribution of references
    pub by_year: HashMap<i32, usize>,

    /// Oldest and newest reference years
    pub year_range: Option<(i32, i32)>,

    /// Top venues referenced
    pub top_venues: Vec<(String, usize)>,
}

impl ReferenceStatistics {
    /// Calculate statistics from a list of papers
    pub fn from_papers(papers: &[PaperSummary]) -> Self {
        let mut by_year: HashMap<i32, usize> = HashMap::new();
        let mut venues: HashMap<String, usize> = HashMap::new();
        let mut min_year: Option<i32> = None;
        let mut max_year: Option<i32> = None;

        for paper in papers {
            if paper.year > 0 {
                *by_year.entry(paper.year).or_insert(0) += 1;
                min_year = Some(min_year.map_or(paper.year, |m| m.min(paper.year)));
                max_year = Some(max_year.map_or(paper.year, |m| m.max(paper.year)));
            }
            if !paper.venue.is_empty() {
                *venues.entry(paper.venue.clone()).or_insert(0) += 1;
            }
        }

        // Sort venues by count and take top 10
        let mut venue_vec: Vec<_> = venues.into_iter().collect();
        venue_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let top_venues: Vec<_> = venue_vec.into_iter().take(10).collect();

        let year_range = match (min_year, max_year) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        };

        Self {
            by_year,
            year_range,
            top_venues,
        }
    }
}

/// Extracted keywords and topics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeywordsData {
    /// Primary research keywords
    pub keywords: Vec<String>,

    /// Research topics/domains
    pub topics: Vec<String>,

    /// Technical terms with definitions
    pub technical_terms: Vec<TechnicalTerm>,

    /// Methods/techniques mentioned
    pub methods: Vec<String>,

    /// Datasets mentioned
    pub datasets: Vec<String>,
}

/// Technical term with optional definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalTerm {
    /// The term
    pub term: String,

    /// Definition (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
}

/// Research field positioning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchContext {
    /// Primary research field
    pub primary_field: String,

    /// Sub-fields
    pub sub_fields: Vec<String>,

    /// Research type (empirical, theoretical, survey, etc.)
    pub research_type: String,

    /// Positioning statement (LLM-generated)
    pub positioning: String,

    /// Related research directions
    pub related_directions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_paper() -> AcademicPaper {
        let mut paper = AcademicPaper::new();
        paper.ss_id = "123".to_string();
        paper.arxiv_id = "2106.09685".to_string();
        paper.title = "Test Paper".to_string();
        paper.abstract_text = "This is a test abstract".to_string();
        paper.citations_count = 100;
        paper
    }

    #[test]
    fn test_paper_summary_from_academic_paper() {
        let paper = create_test_paper();

        let summary = PaperSummary::from_academic_paper(&paper);
        assert_eq!(summary.ss_id, "123");
        assert_eq!(summary.arxiv_id, "2106.09685");
        assert_eq!(summary.title, "Test Paper");
        assert_eq!(summary.citation_count, 100);
    }

    #[test]
    fn test_abstract_truncation() {
        let long_abstract = "a".repeat(600);
        let mut paper = AcademicPaper::new();
        paper.abstract_text = long_abstract;

        let summary = PaperSummary::from_academic_paper(&paper);
        assert!(summary.abstract_snippet.len() < 510);
        assert!(summary.abstract_snippet.ends_with("..."));
    }

    #[test]
    fn test_citation_statistics() {
        let papers = vec![
            PaperSummary {
                year: 2020,
                venue: "NeurIPS".to_string(),
                citation_count: 100,
                title: "Paper 1".to_string(),
                ..Default::default()
            },
            PaperSummary {
                year: 2021,
                venue: "NeurIPS".to_string(),
                citation_count: 50,
                title: "Paper 2".to_string(),
                ..Default::default()
            },
            PaperSummary {
                year: 2020,
                venue: "ICML".to_string(),
                citation_count: 200,
                title: "Paper 3".to_string(),
                ..Default::default()
            },
        ];

        let stats = CitationStatistics::from_papers(&papers);
        assert_eq!(stats.by_year.get(&2020), Some(&2));
        assert_eq!(stats.by_year.get(&2021), Some(&1));
        assert!(!stats.top_venues.is_empty());
        assert_eq!(stats.most_influential[0], "Paper 3");
    }

    #[test]
    fn test_exported_paper_new() {
        let paper = AcademicPaper::new();
        let options = ExportOptions::default();
        let exported = ExportedPaper::new(paper, options);

        assert_eq!(exported.schema_version, EXPORT_SCHEMA_VERSION);
        assert!(exported.citations.is_none());
        assert!(exported.references.is_none());
    }
}
