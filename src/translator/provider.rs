use super::heuristic::Translation;

pub trait TranslatorProvider {
    fn translate(&self, input: &str) -> Translation;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::heuristic::translate_heuristic;

    struct HeuristicProvider;

    impl TranslatorProvider for HeuristicProvider {
        fn translate(&self, input: &str) -> Translation {
            translate_heuristic(input)
        }
    }

    #[test]
    fn provider_trait_can_return_translation() {
        let provider = HeuristicProvider;
        let translation = provider.translate("show errors");

        assert!(!translation.query.filters.must.is_empty());
    }
}
