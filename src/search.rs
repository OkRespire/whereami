use crate::AppState;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn filter_search(state: &mut AppState) {
    state.clients_to_display.clear();

    if state.query.is_empty() {
        state.clients_to_display = state
            .clients
            .iter()
            .filter_map(|client| {
                client
                    .title
                    .as_ref()
                    .map(|title| (client.clone(), title.clone()))
            })
            .collect();
        return;
    }
    let matcher = SkimMatcherV2::default();

    let mut scored_clients = state
        .clients
        .iter()
        .filter_map(|client| {
            let client_title = client.title.as_ref()?;

            if let Some(score) = matcher.fuzzy_match(&client_title, &state.query) {
                Some((score, client.clone(), client_title.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    scored_clients.sort_by(|a, b| b.0.cmp(&a.0));

    state.clients_to_display = scored_clients
        .into_iter()
        .map(|(_, client, names)| (client, names))
        .collect();
}
