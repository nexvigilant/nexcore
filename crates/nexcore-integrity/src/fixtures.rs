//! # KSB Test Fixtures
//!
//! 20 KSB assessment fixtures spanning Bloom levels 1-7 and domains D02-D12.
//! Each fixture has a human_response and an ai_response (~80-120 words).
//!
//! Distribution: 3 at Bloom 1-2, 4 at Bloom 3-4, 3 at Bloom 5, 2 at Bloom 6, 1 at Bloom 7

/// A single KSB test fixture.
pub struct KsbFixture {
    /// KSB identifier
    pub ksb_id: &'static str,
    /// Bloom taxonomy level (1-7)
    pub bloom_level: u8,
    /// Domain ID
    pub domain_id: &'static str,
    /// Human-written response
    pub human_response: &'static str,
    /// AI-generated response
    pub ai_response: &'static str,
}

/// All 20 KSB test fixtures.
pub fn all_fixtures() -> Vec<KsbFixture> {
    vec![
        // === Bloom Level 1 (Remember) — 2 fixtures ===
        KsbFixture {
            ksb_id: "K-D02-001",
            bloom_level: 1,
            domain_id: "D02",
            human_response: "ICH E2A defines serious adverse events as those resulting in death, hospitalization, disability, congenital anomaly, or life-threatening situations. I remember from my training that the timelines are strict — expedited reports for fatal and life-threatening cases must be submitted within 15 calendar days of awareness, while periodic reports follow different schedules depending on the marketing authorization holder's obligations. The regulation also covers what constitutes an 'unexpected' reaction based on the reference safety information. Each region may have slight variations but E2A sets the international baseline that most regulatory bodies reference in their own legislation.",
            ai_response: "ICH E2A provides a comprehensive framework for defining and reporting adverse drug reactions in the post-marketing setting. It establishes clear criteria for seriousness, including death, hospitalization, disability, congenital anomaly, and life-threatening conditions. The guideline specifies expedited reporting timelines of 15 calendar days for serious cases and 7 days for fatal or life-threatening events. Additionally, it defines the concept of expectedness based on the approved product information. The guideline serves as the international standard that regulatory authorities worldwide use as the basis for their pharmacovigilance reporting requirements and safety communication obligations.",
        },
        KsbFixture {
            ksb_id: "K-D03-002",
            bloom_level: 1,
            domain_id: "D03",
            human_response: "A QPPV has to be permanently available — that's the key thing I recall. They're responsible for the pharmacovigilance system, signal detection oversight, and ensuring regulatory compliance across all products. From what I studied, the QPPV must reside in the EU/EEA and have access to the safety database at all times. They sign off on PSURs and RMPs, though in practice a lot of the day-to-day work gets delegated. What sticks with me is that the QPPV carries personal legal liability, which is pretty unusual for a corporate role. The requirements come from EU GVP Module I primarily.",
            ai_response: "The Qualified Person for Pharmacovigilance (QPPV) serves as the single point of accountability for pharmacovigilance activities within the European Economic Area. Their responsibilities include oversight of the pharmacovigilance system master file, signal detection and evaluation processes, and ensuring compliance with regulatory reporting obligations. The QPPV must be permanently and continuously available, maintain access to the safety database, and reside within the EU/EEA. They are responsible for reviewing and approving periodic safety update reports and risk management plans. The regulatory framework governing QPPV requirements is primarily established in EU GVP Module I and the pharmacovigilance legislation.",
        },
        // === Bloom Level 2 (Understand) — 1 fixture ===
        KsbFixture {
            ksb_id: "K-D04-003",
            bloom_level: 2,
            domain_id: "D04",
            human_response: "So an ICSR basically captures the four minimum elements: an identifiable reporter, an identifiable patient, at least one suspect drug, and at least one adverse reaction. The way I understand it, the reporter doesn't need to give their name necessarily — a healthcare professional code or institution can suffice. Same with the patient: initials, age, or even 'elderly female' works. The drug name can be generic or brand. What's tricky in practice is when you get incomplete reports — technically valid with four elements but clinically useless. The MedDRA coding adds another layer of complexity since terms get updated annually.",
            ai_response: "An Individual Case Safety Report (ICSR) represents the fundamental unit of pharmacovigilance data collection. It requires four minimum elements for validity: an identifiable reporter, an identifiable patient, a suspected medicinal product, and a suspected adverse reaction. The reporter identification can include healthcare professional credentials or institutional affiliation. Patient identification may consist of initials, date of birth, age, or gender descriptors. The suspected drug should be identified by brand name, generic name, or active substance. Adverse reactions are coded using the Medical Dictionary for Regulatory Activities (MedDRA) terminology. Complete case processing involves assessment of causality, seriousness, and expectedness.",
        },
        // === Bloom Level 3 (Apply) — 2 fixtures ===
        KsbFixture {
            ksb_id: "S-D08-004",
            bloom_level: 3,
            domain_id: "D08",
            human_response: "When I ran the disproportionality analysis on this drug-event combination, the PRR came out at 3.2 with a chi-squared of 12.4 and n=8. That clearly exceeds the default thresholds (PRR >= 2.0, chi2 >= 3.841, n >= 3). But here's what gave me pause — the ROR confidence interval was wide, from 1.4 to 7.1, because of the small case count. The IC025 was just barely positive at 0.12. So the signal exists statistically, but it's fragile. I'd flag it for clinical review rather than immediate escalation. In my experience, signals with n < 10 need extra scrutiny even when the metrics look strong.",
            ai_response: "The disproportionality analysis yielded a Proportional Reporting Ratio of 3.2, which exceeds the standard threshold of 2.0. The chi-squared value of 12.4 surpasses the critical value of 3.841, and the case count of 8 meets the minimum requirement of 3 cases. The Reporting Odds Ratio confidence interval of 1.4 to 7.1 excludes unity, providing additional statistical evidence. The Information Component lower bound of 0.12 is positive, indicating disproportionate reporting. Based on these metrics, the drug-event combination meets the criteria for signal detection. The signal should be further evaluated through clinical assessment and additional data sources to determine its clinical significance.",
        },
        KsbFixture {
            ksb_id: "S-D10-005",
            bloom_level: 3,
            domain_id: "D10",
            human_response: "To apply the BRAT framework here, I started by listing known benefits: 65% tumor response rate, median PFS of 11 months, improved quality of life scores. Then the risks: hepatotoxicity in 12% of patients (2% grade 3+), neutropenia requiring dose delays in 18%, and this new rhabdomyolysis signal we just detected. The key outcome I had to weigh was whether the rhabdomyolysis signal — currently 4 cases in 15,000 exposed patients — materially changes the benefit-risk profile. Given the oncology indication with limited alternatives, I concluded the profile remains favorable but recommended enhanced monitoring and updated prescribing information.",
            ai_response: "Applying the Benefit-Risk Action Team framework requires systematic evaluation of favorable and unfavorable effects. The benefits include a 65% tumor response rate, median progression-free survival of 11 months, and improved quality of life measures. The risks encompass hepatotoxicity occurring in 12% of patients with 2% experiencing grade 3 or higher severity, neutropenia requiring dose modifications in 18% of patients, and the newly identified rhabdomyolysis signal. The rhabdomyolysis signal consists of 4 cases among 15,000 exposed patients. Given the serious oncology indication and limited therapeutic alternatives, the overall benefit-risk balance remains favorable. Enhanced monitoring and prescribing information updates are recommended as risk minimization measures.",
        },
        // === Bloom Level 4 (Analyze) — 2 fixtures ===
        KsbFixture {
            ksb_id: "S-D08-006",
            bloom_level: 4,
            domain_id: "D08",
            human_response: "Breaking down why this signal might be confounded: the drug is primarily used in elderly patients with multiple comorbidities, and the event — acute kidney injury — has a high background rate in that population. Looking at the Weber effect, the product launched only 18 months ago, so we're still in the early reporting peak. Three of the 11 cases had concomitant NSAIDs, which are known nephrotoxins. Two patients had pre-existing renal impairment. One case had a positive dechallenge but also stopped the NSAID simultaneously. The temporal relationship varies wildly — onset from 3 days to 6 months. This heterogeneity makes me suspect we're seeing confounding rather than a true signal.",
            ai_response: "Analysis of the confounding factors reveals several important considerations for this signal. The target population consists predominantly of elderly patients with multiple comorbidities, representing a group with inherently elevated background rates of acute kidney injury. The Weber effect may contribute to reporting bias, as the product was launched only 18 months ago. Among the 11 reported cases, three involved concomitant use of nonsteroidal anti-inflammatory drugs, which are established nephrotoxins. Two patients had pre-existing renal impairment documented in their medical histories. One case demonstrated positive dechallenge, although concurrent discontinuation of concomitant medication complicates interpretation. The heterogeneous time to onset ranging from 3 days to 6 months suggests multiple contributing factors.",
        },
        KsbFixture {
            ksb_id: "S-D04-007",
            bloom_level: 4,
            domain_id: "D04",
            human_response: "The duplicate detection problem in our safety database is actually more nuanced than it appears. When I analyzed the workflow breakdown, I found three root causes. First, the same case arrives via different channels — the physician reports to us directly, then the patient calls the hotline, and the literature team picks up a published case report. Second, follow-up information sometimes gets entered as new cases rather than amendments because the initial report didn't have enough identifiers. Third — and this was surprising — some duplicate ICSRs are actually different patients with similar demographics at the same site. We were incorrectly deduplicating genuine distinct cases. My recommendation is to implement fuzzy matching with a human review step.",
            ai_response: "Analyzing the duplicate detection challenges in the safety database reveals several systematic issues. First, multi-channel reporting creates duplicate entries when the same adverse event is reported through different pathways, including direct healthcare professional reports, patient hotline contacts, and literature monitoring. Second, insufficient case identifiers in initial reports lead to follow-up information being processed as separate cases rather than amendments to existing records. Third, cases involving different patients with similar demographic profiles at the same investigational site may be incorrectly identified as duplicates. To address these challenges, implementation of fuzzy matching algorithms with configurable similarity thresholds is recommended, combined with a human review step for borderline cases.",
        },
        // === Bloom Level 5 (Evaluate) — 3 fixtures ===
        KsbFixture {
            ksb_id: "B-D10-008",
            bloom_level: 5,
            domain_id: "D10",
            human_response: "Having reviewed both the company's PBRER and the independent assessment, I have to conclude the benefit-risk balance for this antihypertensive is shifting unfavorably. Here's my reasoning: the pivotal trial showed only marginal superiority over existing ACE inhibitors (2mmHg additional reduction), while post-marketing data now reveals a hepatotoxicity signal with a fatality rate of 1 per 50,000 patients. That's not acceptable when equally effective alternatives exist without this risk. The company's argument that the hepatotoxicity is 'manageable with monitoring' ignores the reality that liver function testing compliance in primary care is notoriously poor — maybe 40% adherence after year one. I'd recommend reclassifying to second-line therapy with mandatory LFT monitoring.",
            ai_response: "Evaluation of the benefit-risk profile for this antihypertensive agent requires consideration of both the clinical benefits and emerging safety concerns. The pivotal clinical trial demonstrated a modest 2mmHg additional blood pressure reduction compared to established ACE inhibitors. Post-marketing surveillance has identified a hepatotoxicity signal with a fatality rate of approximately 1 per 50,000 exposed patients. Given the availability of alternative antihypertensive agents with comparable efficacy profiles and without hepatotoxic potential, the current benefit-risk balance warrants reassessment. The feasibility of liver function test monitoring as a risk minimization measure is limited by real-world adherence rates. Reclassification as second-line therapy with mandatory monitoring requirements represents an appropriate regulatory response.",
        },
        KsbFixture {
            ksb_id: "B-D08-009",
            bloom_level: 5,
            domain_id: "D08",
            human_response: "I want to push back on the team's decision to close this signal. Yes, the disproportionality metrics are borderline — PRR 1.8, just below our 2.0 threshold. But I've been tracking this drug-event pair for three consecutive quarters and the trend is consistently upward: PRR went from 1.2 to 1.5 to 1.8. The case quality is also improving — the latest three cases have strong temporal relationships and one has a positive rechallenge, which is the gold standard for causality. In my judgment, applying a rigid threshold cutoff here misses the clinical forest for the statistical trees. The trend trajectory matters more than any single-point metric. I'm recommending we keep this signal under active monitoring.",
            ai_response: "The decision to close the signal based on borderline disproportionality metrics merits careful reconsideration. While the current Proportional Reporting Ratio of 1.8 falls below the standard threshold of 2.0, the temporal trend demonstrates consistent upward progression from 1.2 to 1.5 to 1.8 across three consecutive quarters. The quality of incoming case reports has improved, with recent submissions demonstrating strong temporal relationships between drug exposure and event onset. One case includes positive rechallenge evidence, which represents the strongest level of causality support. Rigid application of threshold-based signal closure criteria may not adequately capture clinically meaningful safety concerns. Continued active monitoring with quarterly trend assessment is recommended.",
        },
        KsbFixture {
            ksb_id: "B-D12-010",
            bloom_level: 5,
            domain_id: "D12",
            human_response: "After auditing this company's signal management process, my honest assessment is it's compliant on paper but dangerously inadequate in practice. They have all the SOPs, the signal detection is automated, and the metrics look clean in their PSMF. But when I drilled into specifics, I found the signal review committee hasn't rejected a signal closure recommendation in three years. That's statistically improbable — it suggests rubber-stamping rather than genuine clinical review. The medical reviewers told me privately they're under time pressure to close signals quickly. The KPI structure rewards throughput over quality. I'm writing this up as a critical finding because the system exists to protect patients, not to process signals efficiently.",
            ai_response: "The audit of the signal management process reveals a discrepancy between documented procedures and operational effectiveness. While standard operating procedures are comprehensive and signal detection systems are properly automated, examination of the signal review committee's decision history reveals concerning patterns. The committee has not rejected a signal closure recommendation in three years, suggesting potential issues with the independence and rigor of the review process. Time pressures on medical reviewers and performance metrics emphasizing processing throughput over quality assessment may be contributing factors. This finding warrants classification as critical, as effective signal management requires genuine clinical evaluation rather than procedural compliance alone.",
        },
        // === Bloom Level 6 (Create) — 2 fixtures ===
        KsbFixture {
            ksb_id: "B-D03-011",
            bloom_level: 6,
            domain_id: "D03",
            human_response: "I've designed a novel PV quality metric that I'm calling the Signal Response Latency Index. Current QMS metrics focus on individual case processing timelines, but nobody measures the gap between signal detection and the first risk minimization action. My SRLI captures this by tracking four timestamps: T0 (statistical signal crosses threshold), T1 (signal validated by committee), T2 (risk assessment completed), T3 (first action implemented, whether label update, DHPC, or study initiation). The SRLI = weighted average of (T1-T0), (T2-T1), (T3-T2) with heavier weight on T3-T2 because that's where I've seen the biggest delays in my career. I tested this retrospectively on our last 30 signals and found that signals with SRLI > 90 days had a 3x higher rate of regulatory authority intervention.",
            ai_response: "A novel pharmacovigilance quality metric has been designed to address the gap in current quality management systems regarding signal-to-action response times. The Signal Response Latency Index measures the temporal progression from statistical signal detection through risk minimization implementation. The metric tracks four key timestamps: T0 representing threshold crossing, T1 for committee validation, T2 for risk assessment completion, and T3 for first action implementation. The SRLI is calculated as a weighted average of the intervals between consecutive timestamps, with increased weighting for the T2-T3 interval reflecting its criticality. Retrospective analysis across 30 historical signals demonstrates that SRLI values exceeding 90 days correlate with a threefold increase in regulatory authority intervention.",
        },
        KsbFixture {
            ksb_id: "B-D08-012",
            bloom_level: 6,
            domain_id: "D08",
            human_response: "My proposal for an integrated signal scoring framework combines three dimensions that are currently evaluated in silos. Dimension 1 is statistical strength — the standard disproportionality metrics, but I'm proposing we use a normalized composite rather than individual thresholds. Dimension 2 is clinical plausibility — scored using a modified Naranjo algorithm where I've replaced the 'previous conclusive reports' question with 'mechanistic evidence from preclinical data.' Dimension 3 is patient impact — I've adapted the CIOMS severity classification into a 5-point scale weighted by reversibility. The three dimensions feed into a single Priority Score via a Hill equation — yes, the same cooperativity function from enzyme kinetics — because I want signals that score high on ALL three dimensions to get disproportionately elevated priority. Partial signals (strong statistics but weak plausibility) should not trigger the same urgency.",
            ai_response: "An integrated signal scoring framework has been developed combining three evaluation dimensions currently assessed independently. The statistical strength dimension incorporates normalized composite disproportionality metrics rather than individual threshold criteria. The clinical plausibility dimension utilizes a modified Naranjo causality algorithm incorporating mechanistic evidence from preclinical data. The patient impact dimension adapts the CIOMS severity classification into a five-point scale weighted by outcome reversibility. These three dimensions are integrated through a Hill equation cooperative function, producing a unified Priority Score that disproportionately elevates signals demonstrating strength across all dimensions simultaneously. This cooperative approach ensures that signals with partial evidence do not receive equivalent prioritization to those with comprehensive supporting evidence.",
        },
        // === Bloom Level 7 (Meta-Create) — 1 fixture ===
        KsbFixture {
            ksb_id: "B-D10-013",
            bloom_level: 7,
            domain_id: "D10",
            human_response: "I want to articulate a theoretical framework for what I'm calling Adaptive Pharmacovigilance — the idea that PV systems should self-calibrate based on accumulated evidence rather than relying on fixed regulatory thresholds. My framework has three axioms. Axiom 1: Detection sensitivity should be inversely proportional to therapeutic alternatives. A rare-disease orphan drug deserves wider tolerance than a me-too antihypertensive because the cost of a false positive (withdrawal of the only treatment) is catastrophic. Axiom 2: Threshold adaptation should follow Bayesian updating — each confirmed signal shifts the prior for related drug-event combinations in the same pharmacological class. Axiom 3: The asymmetry of harm requires that downward threshold adjustments (becoming more sensitive) should be irreversible without explicit regulatory review, while upward adjustments can be routine. This framework challenges the current one-size-fits-all approach but I believe it's necessary because we're drowning in signals for well-characterized drugs while potentially missing novel risks for drugs with thin safety databases.",
            ai_response: "An adaptive pharmacovigilance framework proposes that safety monitoring systems should self-calibrate based on accumulated evidence rather than maintaining fixed regulatory thresholds. The framework establishes three foundational axioms. The first axiom posits that detection sensitivity should be inversely proportional to the availability of therapeutic alternatives, reflecting the differential costs of false positive signals across therapeutic areas. The second axiom proposes Bayesian updating of detection thresholds, where confirmed signals modify the prior probability estimates for related drug-event combinations within the same pharmacological class. The third axiom establishes asymmetric threshold adjustment, requiring that increased sensitivity adjustments be irreversible without explicit regulatory review while decreased sensitivity adjustments may follow routine procedures. This framework addresses limitations in current uniform threshold approaches for pharmacovigilance systems.",
        },
        // === Additional fixtures to reach 20 ===
        KsbFixture {
            ksb_id: "K-D02-014",
            bloom_level: 2,
            domain_id: "D02",
            human_response: "The difference between a PSUR and a PBRER confused me at first, but now I get it. A PSUR was the old format — basically a periodic summary of safety data organized by system organ class. The PBRER replaced it under ICH E2C(R2) and the key evolution is the explicit benefit-risk evaluation framework. It's not just 'here are the adverse events' anymore, it's 'here's how the benefit-risk balance has evolved.' In practice, the PBRER includes an integrated benefit-risk analysis section, signal evaluation summaries, and a conclusion on whether the marketing authorization conditions remain appropriate. I think of it as: PSUR was a safety report, PBRER is a benefit-risk argument.",
            ai_response: "The transition from Periodic Safety Update Reports to Periodic Benefit-Risk Evaluation Reports represents a fundamental evolution in pharmacovigilance periodic reporting. PSURs provided periodic summaries of safety data organized by system organ class, focusing primarily on adverse event documentation. PBRERs, established under ICH E2C(R2), incorporate an explicit benefit-risk evaluation framework that extends beyond safety data compilation. The PBRER format includes integrated benefit-risk analysis sections, comprehensive signal evaluation summaries, and conclusions regarding the continued appropriateness of marketing authorization conditions. This evolution reflects the regulatory shift from purely safety-focused periodic reporting to systematic benefit-risk assessment throughout the product lifecycle.",
        },
        KsbFixture {
            ksb_id: "S-D03-015",
            bloom_level: 3,
            domain_id: "D03",
            human_response: "Setting up the signal detection configuration for our new oncology product, I had to make practical decisions. I chose sensitive thresholds (PRR >= 1.5, n >= 2, IC025 > -0.5) because it's a novel mechanism of action with limited safety data — maybe 3,000 patients exposed so far. I set the detection frequency to monthly rather than quarterly, which is more resource-intensive but appropriate given the early lifecycle stage. For the comparator, I used the full drug class rather than individual competitor products because our case volumes are too low for meaningful individual comparisons. The one thing I agonized over was whether to include medically confirmed cases only or all reports — I went with all reports but flagged a separate confirmed-only analysis run.",
            ai_response: "Configuration of the signal detection system for the new oncology product requires consideration of product-specific factors. Sensitive detection thresholds have been selected, including PRR greater than or equal to 1.5, minimum case count of 2, and IC025 greater than -0.5. These thresholds are appropriate given the novel mechanism of action and limited safety database of approximately 3,000 exposed patients. Monthly detection frequency has been implemented rather than quarterly intervals to ensure timely identification of emerging safety concerns during the early post-marketing period. The comparator dataset utilizes the full pharmacological drug class rather than individual products due to insufficient individual case volumes for meaningful disproportionality analysis.",
        },
        KsbFixture {
            ksb_id: "S-D12-016",
            bloom_level: 4,
            domain_id: "D12",
            human_response: "Analyzing the root cause of the inspection finding, I traced the problem back to a gap between two SOPs. SOP-PV-003 covers initial case receipt and triage, ending at 'case forwarded to medical review.' SOP-PV-007 covers medical assessment starting at 'case assigned to medical reviewer.' But there's a liminal space — the handoff queue — that neither SOP covers. Cases sit in this queue anywhere from 2 to 14 days with no oversight, no escalation triggers, and no tracking metrics. The inspector caught three cases that exceeded the 15-day reporting deadline specifically because they languished in this handoff gap. My root cause classification: systemic process gap, not human error. The fix requires a bridging SOP or, better yet, merging the two existing ones.",
            ai_response: "Root cause analysis of the inspection finding reveals a systematic process gap between two standard operating procedures. SOP-PV-003 governs initial case receipt and triage processes, concluding at case forwarding to medical review. SOP-PV-007 addresses medical assessment beginning at case assignment to a medical reviewer. The interval between these procedures — the handoff queue — lacks defined oversight mechanisms, escalation triggers, and tracking metrics. Cases experience delays of 2 to 14 days during this unmonitored transition period. Three specific cases exceeded the 15-day regulatory reporting deadline due to delays within this handoff gap. The root cause is classified as a systemic process gap rather than individual human error, requiring procedural remediation.",
        },
        KsbFixture {
            ksb_id: "B-D04-017",
            bloom_level: 5,
            domain_id: "D04",
            human_response: "I need to evaluate whether our AI-assisted MedDRA coding tool is actually improving accuracy or just creating a false sense of security. Looking at the data critically: coding concordance with expert human coders is 94% at the PT level, which sounds impressive. But when I stratified by case complexity, simple cases hit 99% concordance while complex multi-system cases drop to 78%. That 78% is worse than our junior coders achieve (82%). The AI excels at pattern matching for common drug-event pairs but struggles with rare presentations, combination toxicities, and cases where the narrative requires clinical interpretation rather than keyword matching. My verdict: useful as a first-pass screening tool for straightforward cases, but it needs human oversight for any case involving more than two system organ classes.",
            ai_response: "Evaluation of the AI-assisted MedDRA coding tool reveals important performance stratification across case complexity levels. Overall coding concordance with expert human coders reaches 94% at the Preferred Term level. However, stratification by case complexity demonstrates significant variation: simple cases achieve 99% concordance while complex multi-system cases yield 78% concordance. The 78% concordance for complex cases falls below the 82% achieved by junior human coders. The AI system demonstrates strength in pattern recognition for common drug-event combinations but exhibits limitations with rare clinical presentations, combination toxicities, and cases requiring narrative clinical interpretation. The tool is recommended for first-pass screening of straightforward cases with mandatory human oversight for complex multi-system cases.",
        },
        KsbFixture {
            ksb_id: "B-D02-018",
            bloom_level: 2,
            domain_id: "D02",
            human_response: "Good Pharmacovigilance Practice, or GVP, is how I understand the EU's operational framework for drug safety monitoring. It's organized into modules — I can never remember all the numbers, but the ones that come up most in my work are Module I (PV systems and their quality), Module VI (management and reporting of adverse reactions), Module IX (signal management), and Module XVI (risk minimization). What I find useful is that GVP bridges the gap between legislation (the directive and regulation) and actual practice. The legislation says 'thou shalt do pharmacovigilance' and GVP says 'here's how.' Each module has been updated at least once since original publication, which makes version tracking a headache.",
            ai_response: "Good Pharmacovigilance Practice represents the European Union's operational framework for implementing pharmacovigilance obligations established by legislation. The framework is organized into numbered modules addressing specific aspects of pharmacovigilance activities. Key modules include Module I covering pharmacovigilance systems and quality management, Module VI addressing adverse reaction management and reporting, Module IX governing signal management processes, and Module XVI focusing on risk minimization measures. GVP provides practical implementation guidance that bridges the gap between legislative requirements and operational activities. The modules undergo periodic revision to reflect evolving regulatory expectations and scientific knowledge. Understanding GVP is essential for organizations conducting pharmacovigilance activities within the European regulatory framework.",
        },
        KsbFixture {
            ksb_id: "S-D10-019",
            bloom_level: 3,
            domain_id: "D10",
            human_response: "For this benefit-risk assessment I need to construct the value tree. On the benefit side: primary efficacy endpoint (HbA1c reduction of 1.2% — clinically meaningful for a diabetes drug), weight loss effect (average 3.5kg — a genuine bonus since most diabetes drugs cause weight gain), and cardiovascular outcome data from the CV trial showing 14% relative risk reduction in MACE. On the risk side: the established GI tolerability issues (nausea in 25%, mostly transient), the pancreatitis signal (rare but serious, 3 per 10,000 patient-years), and this new thyroid concern we're investigating. I'm using multi-criteria decision analysis weights of 40% efficacy, 25% CV benefit, 20% GI tolerability, 10% pancreatitis, 5% thyroid.",
            ai_response: "Construction of the benefit-risk value tree for this diabetes agent identifies key favorable and unfavorable effects. Benefits include the primary efficacy endpoint of 1.2% HbA1c reduction, a weight loss effect averaging 3.5 kilograms, and cardiovascular outcome data demonstrating a 14% relative risk reduction in major adverse cardiovascular events. Risks encompass gastrointestinal tolerability issues including nausea in 25% of patients, a pancreatitis signal occurring at a rate of 3 per 10,000 patient-years, and an emerging thyroid safety concern under investigation. Multi-criteria decision analysis weighting assigns 40% to primary efficacy, 25% to cardiovascular benefit, 20% to gastrointestinal tolerability, 10% to pancreatitis risk, and 5% to the thyroid concern.",
        },
        KsbFixture {
            ksb_id: "B-D08-020",
            bloom_level: 6,
            domain_id: "D08",
            human_response: "I've developed a prototype for what I'm calling the Temporal Signal Fingerprint — a visual and quantitative method for characterizing signal evolution patterns. Instead of just tracking whether a signal is above or below threshold, the TSF captures the shape of the signal's trajectory over time: steep rise, plateau, oscillation, or gradual climb. I encode this as a 12-month rolling vector of delta-PRR values (change in PRR month over month). Clustering these vectors with k-means revealed four distinct signal archetypes in our database: 'flash' signals (rapid rise, rapid fall — usually stimulated reporting), 'crescendo' signals (steady accumulation — true safety issues), 'seasonal' signals (cyclical — often confounded by indication), and 'plateau' signals (persistent moderate level — usually established and well-characterized risks). The practical value is triage: flash signals can be deprioritized, crescendos need immediate attention.",
            ai_response: "A novel Temporal Signal Fingerprint methodology has been developed for characterizing signal evolution patterns through quantitative trajectory analysis. Rather than binary threshold-based assessment, the TSF captures signal trajectory shapes over time including steep rises, plateaus, oscillations, and gradual climbs. The methodology encodes signals as 12-month rolling vectors of delta-PRR values representing month-over-month changes. Cluster analysis using k-means identified four distinct signal archetypes: flash signals characterized by rapid rise and fall patterns typically associated with stimulated reporting, crescendo signals demonstrating steady accumulation indicative of genuine safety concerns, seasonal signals showing cyclical patterns often confounded by indication, and plateau signals exhibiting persistent moderate levels representing well-characterized risks.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment;

    #[test]
    fn test_fixture_count() {
        assert_eq!(all_fixtures().len(), 20);
    }

    #[test]
    fn test_bloom_level_distribution() {
        let fixtures = all_fixtures();
        let count_at = |level: u8| fixtures.iter().filter(|f| f.bloom_level == level).count();
        assert_eq!(count_at(1), 2); // 001, 002
        assert_eq!(count_at(2), 3); // 003, 014, 018
        assert_eq!(count_at(3), 4); // 004, 005, 015, 019
        assert_eq!(count_at(4), 3); // 006, 007, 016
        assert_eq!(count_at(5), 4); // 008, 009, 010, 017
        assert_eq!(count_at(6), 3); // 011, 012, 020
        assert_eq!(count_at(7), 1); // 013
        let total: usize = (1..=7).map(count_at).sum();
        assert_eq!(total, 20, "Expected 20 total fixtures");
    }

    #[test]
    fn test_all_bloom_levels_valid() {
        for fixture in all_fixtures() {
            assert!(
                fixture.bloom_level >= 1 && fixture.bloom_level <= 7,
                "{}: bloom_level={}",
                fixture.ksb_id,
                fixture.bloom_level
            );
        }
    }

    #[test]
    fn test_all_domain_ids_valid() {
        let valid_domains = ["D02", "D03", "D04", "D08", "D10", "D12"];
        for fixture in all_fixtures() {
            assert!(
                valid_domains.contains(&fixture.domain_id),
                "{}: invalid domain {}",
                fixture.ksb_id,
                fixture.domain_id
            );
        }
    }

    #[test]
    fn test_responses_substantial() {
        for fixture in all_fixtures() {
            let human_words = fixture.human_response.split_whitespace().count();
            let ai_words = fixture.ai_response.split_whitespace().count();
            assert!(
                human_words >= 60,
                "{}: human response only {} words",
                fixture.ksb_id,
                human_words
            );
            assert!(
                ai_words >= 60,
                "{}: AI response only {} words",
                fixture.ksb_id,
                ai_words
            );
        }
    }

    #[test]
    fn test_human_responses_assessable() {
        for fixture in all_fixtures() {
            let result = assessment::assess_ksb_response(
                fixture.human_response,
                fixture.bloom_level,
                Some(fixture.domain_id),
            );
            assert!(
                result.is_ok(),
                "{}: human response failed assessment: {:?}",
                fixture.ksb_id,
                result
            );
        }
    }

    #[test]
    fn test_ai_responses_assessable() {
        for fixture in all_fixtures() {
            let result = assessment::assess_ksb_response(
                fixture.ai_response,
                fixture.bloom_level,
                Some(fixture.domain_id),
            );
            assert!(
                result.is_ok(),
                "{}: AI response failed assessment: {:?}",
                fixture.ksb_id,
                result
            );
        }
    }
}
