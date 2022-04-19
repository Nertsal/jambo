use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct GoogleSheetConfig {
    pub sheet_id: String,
    display_luck: bool,
    cell_format: GoogleSheetCellFormat,
}

#[derive(Clone, Serialize, Deserialize)]
struct GoogleSheetCellFormat {
    color_queued: Option<google_sheets4::api::Color>,
    color_current: Option<google_sheets4::api::Color>,
    color_skipped: Option<google_sheets4::api::Color>,
    color_played: Option<google_sheets4::api::Color>,
}

impl GamejamBot {
    pub async fn save_sheets(&self) -> google_sheets4::Result<()> {
        use google_sheets4::api::*;

        // Headers
        let mut rows = Vec::new();
        let mut values = vec!["Game link".to_owned(), "Author".to_owned()];
        if self
            .config
            .google_sheet_config
            .as_ref()
            .unwrap()
            .display_luck
        {
            values.push("Luck".to_owned());
        }
        rows.push(self.values_to_row_data(
            values,
            Some(CellFormat {
                text_format: Some(TextFormat {
                    bold: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ));

        // Current game
        let current_game = match &self.state.current_state {
            GameJamState::Playing { game } | GameJamState::Waiting { game, .. } => Some(game),
            _ => None,
        };
        if let Some(game) = current_game {
            rows.push(self.values_to_row_data(
                self.game_to_values(game),
                self.game_to_format(GameType::Current),
            ));
        }

        // Queued games
        for game in self.state.submissions.queue.get_queue() {
            let mut values = self.game_to_values(game);
            if let Some(sheet_config) = &self.config.google_sheet_config {
                if sheet_config.display_luck {
                    values.push(
                        self.state
                            .raffle_weights
                            .get(&game.link)
                            .copied()
                            .unwrap_or(self.config.raffle_default_weight)
                            .to_string(),
                    )
                }
            }
            rows.push(self.values_to_row_data(values, self.game_to_format(GameType::Queued)));
        }

        for game in &self.state.submissions.skipped {
            rows.push(self.values_to_row_data(
                self.game_to_values(game),
                self.game_to_format(GameType::Skipped),
            ));
        }
        for game in &self.state.submissions.played_games {
            rows.push(self.values_to_row_data(
                self.game_to_values(game),
                self.game_to_format(GameType::Played),
            ));
        }

        let update_values = BatchUpdateSpreadsheetRequest {
            requests: Some(vec![
                Request {
                    update_sheet_properties: Some(UpdateSheetPropertiesRequest {
                        properties: Some(SheetProperties {
                            grid_properties: Some(GridProperties {
                                frozen_row_count: Some(1),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        fields: Some("gridProperties.frozenRowCount".to_owned()),
                    }),
                    ..Default::default()
                },
                Request {
                    repeat_cell: Some(RepeatCellRequest {
                        fields: Some("*".to_owned()),
                        range: Some(GridRange {
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Request {
                    update_cells: Some(UpdateCellsRequest {
                        rows: Some(rows),
                        fields: Some("*".to_owned()),
                        start: Some(GridCoordinate {
                            row_index: Some(0),
                            column_index: Some(0),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let result = self
            .hub
            .as_ref()
            .unwrap()
            .spreadsheets()
            .batch_update(
                update_values,
                &self.config.google_sheet_config.as_ref().unwrap().sheet_id,
            )
            .add_scope(Scope::Spreadsheet)
            .doit()
            .await;
        result.map(|_| ())
    }

    fn game_to_values(&self, game: &Submission) -> Vec<String> {
        let mut authors = game.authors.iter();
        let mut game_authors = authors.next().unwrap().to_owned();
        for author in authors {
            game_authors.push_str(", ");
            game_authors.push_str(author);
        }
        vec![game.link.clone(), game_authors]
    }

    fn game_to_format(&self, game_type: GameType) -> Option<google_sheets4::api::CellFormat> {
        use google_sheets4::api::*;
        let cell_format = &self
            .config
            .google_sheet_config
            .as_ref()
            .unwrap()
            .cell_format;
        let color = match game_type {
            GameType::Queued => cell_format.color_queued.clone(),
            GameType::Current => cell_format.color_current.clone(),
            GameType::Skipped => cell_format.color_skipped.clone(),
            GameType::Played => cell_format.color_played.clone(),
        };
        Some(CellFormat {
            background_color: color,
            ..Default::default()
        })
    }

    fn values_to_row_data(
        &self,
        values: Vec<String>,
        user_entered_format: Option<google_sheets4::api::CellFormat>,
    ) -> google_sheets4::api::RowData {
        use google_sheets4::api::*;
        let mut cells = Vec::with_capacity(values.len());
        for value in values {
            cells.push(CellData {
                user_entered_value: Some(ExtendedValue {
                    string_value: Some(value),
                    ..Default::default()
                }),
                user_entered_format: user_entered_format.clone(),
                ..Default::default()
            });
        }
        RowData {
            values: Some(cells),
        }
    }
}
