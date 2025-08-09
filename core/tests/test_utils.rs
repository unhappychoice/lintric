#![allow(unused_macros)]

#[macro_export]
macro_rules! assert_analysis_results_match {
    ($result_ar:expr, $expected_ar:expr) => {
        assert_eq!(
            $result_ar.overall_complexity_score,
            $expected_ar.overall_complexity_score,
            "Overall complexity score mismatch for file_path: {}",
            $result_ar.file_path
        );

        assert_eq!(
            $result_ar.line_metrics.len(),
            $expected_ar.line_metrics.len(),
            "Number of line metrics do not match for file_path: {}",
            $result_ar.file_path
        );

        for result_lm in $result_ar.line_metrics.iter() {
            let expected_lm = $expected_ar.line_metrics
                .iter()
                .find(|x| x.line_number == result_lm.line_number)
                .unwrap_or_else(|| {
                    panic!(
                        "No matching expected line metrics found for line {} in file_path: {}",
                        result_lm.line_number, $result_ar.file_path
                    )
                });
            assert_eq!(
                result_lm,
                expected_lm,
                "Line metrics mismatch for line {} in file_path: {}",
                result_lm.line_number, $result_ar.file_path
            );
        }
    };
}
