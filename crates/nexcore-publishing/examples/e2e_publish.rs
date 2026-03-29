//! End-to-end publishing demo: build a DOCX → parse → EPUB → validate → KDP check.

use std::io::{Cursor, Write as _};

fn main() {
    // === Step 1: Build a sample DOCX in memory ===
    println!("=== Step 1: Building test DOCX ===");
    let docx_bytes = build_sample_docx();
    let docx_path = "/tmp/pv-for-nexvigilants.docx";
    std::fs::write(docx_path, &docx_bytes).unwrap();
    println!("  Written: {} ({} bytes)", docx_path, docx_bytes.len());

    // === Step 2: Parse the DOCX ===
    println!("\n=== Step 2: Parsing DOCX ===");
    let doc = nexcore_publishing::docx::DocxDocument::from_file(docx_path).unwrap();
    println!("  Paragraphs: {}", doc.paragraph_count());
    println!("  Words: {}", doc.word_count());
    println!("  Title: {:?}", doc.properties.title);
    println!("  Author: {:?}", doc.properties.creator);

    // === Step 3: Enrich metadata ===
    println!("\n=== Step 3: Metadata ===");
    let mut metadata = doc.to_metadata();
    metadata.description =
        Some("A practical guide to pharmacovigilance for intelligent beginners.".into());
    metadata.subjects = vec![
        "Pharmacovigilance".into(),
        "Drug Safety".into(),
        "Healthcare".into(),
    ];
    metadata.publisher = Some("NexVigilant Press".into());
    metadata.date = Some("2026-03-28".into());
    metadata.rights = Some("Copyright 2026 NexVigilant. All rights reserved.".into());
    let issues = metadata.validate();
    println!("  Title: {}", metadata.title);
    println!("  Author: {}", metadata.primary_author());
    println!("  Language: {}", metadata.language);
    println!(
        "  Validation: {}",
        if issues.is_empty() {
            "PASS".to_string()
        } else {
            issues.join(", ")
        }
    );

    // === Step 4: Split into chapters ===
    println!("\n=== Step 4: Chapter splitting ===");
    let chapters = nexcore_publishing::chapter::split_into_chapters(
        &doc.paragraphs,
        nexcore_publishing::chapter::HeadingLevel::H1,
    );
    for ch in &chapters {
        println!("  [{}] {} ({} words)", ch.order, ch.title, ch.word_count());
    }
    let total_words: usize = chapters.iter().map(|c| c.word_count()).sum();
    println!(
        "  Total: {} chapters, {} words",
        chapters.len(),
        total_words
    );

    // === Step 5: Generate EPUB ===
    println!("\n=== Step 5: Generating EPUB ===");
    let epub_bytes = nexcore_publishing::epub::generate_epub(
        &metadata,
        &chapters,
        None,
        &nexcore_publishing::epub::EpubOptions::default(),
    )
    .unwrap();
    let epub_path = "/tmp/pv-for-nexvigilants.epub";
    std::fs::write(epub_path, &epub_bytes).unwrap();
    println!(
        "  Written: {} ({} bytes / {:.1} KB)",
        epub_path,
        epub_bytes.len(),
        epub_bytes.len() as f64 / 1024.0
    );

    // === Step 6: Validate EPUB ===
    println!("\n=== Step 6: EPUB Validation ===");
    let validation = nexcore_publishing::validate::validate_epub_bytes(&epub_bytes).unwrap();
    println!("{}", validation.to_report());

    // === Step 7: KDP Compliance ===
    println!("=== Step 7: KDP Compliance ===");
    let kdp = nexcore_publishing::kindle::check_kdp_compliance(&metadata, &chapters, None);
    println!("{}", kdp.to_report());

    // === Step 8: List EPUB contents ===
    println!("=== Step 8: EPUB Archive Contents ===");
    let cursor = Cursor::new(&epub_bytes);
    let archive = zip::ZipArchive::new(cursor).unwrap();
    for i in 0..archive.len() {
        if let Some(name) = archive.name_for_index(i) {
            println!("  {}", name);
        }
    }

    println!("\n========================================");
    println!("  SUCCESS: EPUB ready at {}", epub_path);
    println!("========================================");
}

fn build_sample_docx() -> Vec<u8> {
    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Content types
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/></Types>"#,
    )
    .unwrap();

    // Relationships
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>"#,
    )
    .unwrap();

    // Document content — 5 chapters of PV content
    zip.start_file("word/document.xml", opts).unwrap();
    let content = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Introduction to Pharmacovigilance</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Pharmacovigilance is the science and activities relating to the detection, assessment, understanding, and prevention of adverse effects or any other drug-related problem. It plays a critical role in ensuring patient safety throughout the lifecycle of a medicinal product. From the moment a drug enters clinical trials to long after it reaches the market, pharmacovigilance professionals monitor, evaluate, and act on safety signals to protect public health. The discipline has evolved significantly since the thalidomide tragedy of the 1960s, which highlighted the devastating consequences of inadequate drug safety monitoring.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>The World Health Organization defines pharmacovigilance as the science relating to the collection, detection, assessment, monitoring, and prevention of adverse effects with pharmaceutical products. This definition encompasses a broad range of activities including individual case safety report processing, signal detection, risk management, and regulatory compliance. Modern pharmacovigilance leverages computational methods, large databases, and artificial intelligence to enhance signal detection capabilities beyond what manual review alone can achieve.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Signal Detection Methods</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Signal detection is the cornerstone of pharmacovigilance. A safety signal is information that arises from one or multiple sources which suggests a new potentially causal association, or a new aspect of a known association, between an intervention and an event or set of related events that is judged to be of sufficient likelihood to justify verificatory action. The four primary disproportionality methods are the Proportional Reporting Ratio, Reporting Odds Ratio, Information Component, and Empirical Bayesian Geometric Mean. Each method has distinct statistical properties and appropriate use cases depending on the size and quality of the underlying spontaneous reporting database.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>The Proportional Reporting Ratio compares the proportion of a specific adverse event for a drug of interest with the proportion of that event for all other drugs in the database. A PRR greater than 2 with at least 3 cases and a chi-squared value greater than 4 suggests a potential signal worthy of further investigation. The Reporting Odds Ratio is calculated similarly to an odds ratio in epidemiology, comparing the odds of a specific event occurring with a specific drug versus all other drugs in the reporting system. Both measures are frequentist in nature and can be computed from standard two-by-two contingency tables derived from spontaneous reporting databases such as FAERS or EudraVigilance.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Causality Assessment</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Once a signal is detected, the next critical step is causality assessment, which evaluates whether the drug actually caused the observed adverse event rather than being a coincidental temporal association. The two most widely used standardized methods are the Naranjo algorithm and the WHO-UMC system. The Naranjo algorithm employs a structured questionnaire of 10 items to score the probability of an adverse drug reaction on a scale ranging from definite through probable and possible to doubtful. Each question addresses key pharmacological and clinical considerations including temporal relationship, dechallenge, rechallenge, and the availability of alternative explanations for the observed adverse event.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Bradford Hill criteria provide a more comprehensive epidemiological framework for evaluating causality at the population level, considering nine factors including strength of association, consistency across studies, specificity, temporality, biological gradient (dose-response), plausibility, coherence with known biology, experimental evidence from controlled studies, and analogy with similar drug-event pairs. These criteria are not absolute requirements but rather guidelines that help assess the cumulative weight of evidence for a causal relationship between drug exposure and adverse health outcomes. Understanding these frameworks is essential for any pharmacovigilance professional making causality determinations.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Regulatory Reporting</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Regulatory reporting is the formal mechanism by which drug safety information is communicated from marketing authorization holders to health authorities around the world. Individual Case Safety Reports must be submitted within strict regulatory timelines that vary by jurisdiction but typically require submission within 15 calendar days for serious and unexpected adverse reactions, and 90 calendar days for non-serious reactions. The ICH E2B R3 format provides a standardized electronic structure for transmitting these reports between pharmaceutical companies, regulatory authorities, and the World Health Organization, enabling global harmonization of safety data exchange and reducing duplication of effort across regulatory systems.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Periodic safety reports such as the Periodic Safety Update Report and the Periodic Benefit-Risk Evaluation Report provide comprehensive cumulative assessments of a medicinal products evolving safety profile at defined regular intervals throughout its marketed lifecycle. These structured reports include a thorough review of all available safety data from clinical trials, spontaneous reports, literature, and other sources, along with an updated evaluation of the overall benefit-risk balance and a detailed description of any risk minimization measures that have been implemented or are being proposed to address identified or potential safety concerns. Marketing Authorization Holders bear primary responsibility for maintaining accurate, complete, and continuously updated safety databases that support these regulatory obligations.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Risk Management</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Risk management in pharmacovigilance involves a comprehensive structured approach to systematically identifying, characterizing, preventing, and minimizing risks associated with the use of medicinal products throughout their entire lifecycle from authorization through post-marketing surveillance. The Risk Management Plan is a foundational regulatory document that describes the known and potential safety profile of a product, outlines the specific pharmacovigilance activities planned to further identify and characterize both known and emerging risks, and provides detailed descriptions of the risk minimization measures to be implemented to reduce the frequency or severity of adverse reactions in clinical practice.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Additional risk minimization measures that go beyond routine measures such as the Summary of Product Characteristics and Package Leaflet may include targeted educational materials and training programs for healthcare professionals and patients, controlled access or restricted distribution programs, mandatory pregnancy prevention programs for teratogenic medications, patient registries designed to collect long-term safety outcomes data, and requirements for specific laboratory monitoring or clinical assessments before and during treatment. The effectiveness of all risk minimization measures, whether routine or additional, should be evaluated regularly through carefully designed effectiveness studies to ensure they are achieving their intended public health objectives and to make evidence-based adjustments as needed.</w:t></w:r></w:p>"#,
        r#"</w:body></w:document>"#,
    );
    zip.write_all(content.as_bytes()).unwrap();

    // Core properties
    zip.start_file("docProps/core.xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Pharmacovigilance For NexVigilants</dc:title><dc:creator>Matthew Campion, PharmD</dc:creator><dc:language>en</dc:language></cp:coreProperties>"#,
    )
    .unwrap();

    zip.finish().unwrap();
    buf
}
