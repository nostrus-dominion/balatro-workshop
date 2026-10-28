return {
  ["GAME"] = {
    ["dollars"] = 125,
    ["round"] = 8,
    ["win_ante"] = 8,
    ["round_resets"] = { ["ante"] = 3, ["hands"] = 4, ["discards"] = 3, ["reroll_cost"] = 5 },
    ["shop"] = { ["joker_max"] = 2 },
    ["probabilities"] = { ["normal"] = 1 },
    ["bankrupt_at"] = 0,
  },
  ["cardAreas"] = {
    ["jokers"] = { ["config"] = { ["card_limit"] = 5 }, ["cards"] = {} },
    ["consumeables"] = { ["config"] = { ["card_limit"] = 2 }, ["cards"] = {} },
    ["hand"] = { ["config"] = { ["card_limit"] = 8 }, ["cards"] = {} },
  },
  ["unknown_mod_data"] = { [1] = "numeric key", ["1"] = "string key", [9] = 1.2345678901234567e100 },
}
