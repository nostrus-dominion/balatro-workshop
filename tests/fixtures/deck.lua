return {
  GAME = { dollars = 125, starting_deck_size = 5 },
  cardAreas = {
    deck = { config = { card_limit = 4 }, cards = {
      [10] = {
        playing_card = 4, save_fields = { card = "C_9", center = "c_base" },
        base = { suit = "Clubs", value = "9", name = "9 of Clubs", id = 9, nominal = 9, face_nominal = 0 },
        ability = { set = "Default", name = "Default Base", effect = "Base" },
      },
      [1] = {
        playing_card = 1, sort_id = 7, rank = 1, label = "Base Card",
        save_fields = { card = "H_2", center = "c_base", unknown = "keep" },
        base = {
          suit = "Hearts", value = "2", name = "2 of Hearts", id = 2, nominal = 2, face_nominal = 0,
          original_value = "2", times_played = 1, suit_nominal = 0.03, suit_nominal_original = 0.003,
          colour = { [1] = 1, [2] = 0, [3] = 0, [4] = 1 },
        },
        ability = {
          set = "Default", name = "Default Base", effect = "Base", perma_bonus = 15,
          played_this_ante = true, forced_selection = true, custom = { message = "keep" },
          bonus = 0, mult = 0, h_x_mult = 0, h_dollars = 0, p_dollars = 0, x_mult = 1,
        },
        edition = { foil = true, chips = 50 }, unknown = { [3] = "keep" },
      },
      ["1"] = { playing_card = 9000 },
      [0] = { playing_card = 9001 },
      [2] = { ability = { set = "Joker" } },
    } },
    hand = { cards = {
      [1] = {
        playing_card = 2, save_fields = { card = "D_Q", center = "m_lucky" }, label = "Lucky Card",
        base = { suit = "Diamonds", value = "Queen", name = "Queen of Diamonds", id = 12, nominal = 10, face_nominal = 0.2 },
        ability = { set = "Enhanced", name = "Lucky Card", effect = "Lucky Card", order = 9, mult = 20, p_dollars = 20 },
        seal = "Red",
      },
    } },
    discard = { cards = {
      [1] = { playing_card = 3, save_fields = { card = "S_K", center = "m_stone" },
        base = { suit = "Spades", value = "King", name = "King of Spades", id = 13, nominal = 10, face_nominal = 0.3 },
        ability = { set = "Enhanced", name = "Stone Card", effect = "Stone Card", order = 7, bonus = 50 },
        seal = "Gold" },
    } },
    play = { cards = {
      [1] = { playing_card = 5, save_fields = { card = "S_5", center = "c_base" },
        base = { suit = "Spades", value = "5", name = "5 of Spades", id = 5, nominal = 5, face_nominal = 0 },
        ability = { set = "Default", name = "Default Base", effect = "Base" } },
    } },
    jokers = { config = { card_limit = 5 }, cards = {
      [1] = { save_fields = { center = "j_joker" }, ability = { set = "Joker", name = "Joker" } },
    } },
    pack_cards = { cards = { [1] = { playing_card = 50 } } },
    shop_jokers = { cards = { [1] = { playing_card = 51 } } },
  },
}
