-- Synthetic editor fixture: six owned Jokers, not a playable Balatro run.
return {
  GAME = {dollars = 125, round = 8},
  cardAreas = {
    jokers = {
      config = {card_limit = 6, card_count = 6, temp_limit = 6, custom = 'preserve'},
      cards = {
        [6] = {ability = {name = 'Blueprint', set = 'Joker', eternal = true}, save_fields = {center = 'j_blueprint'}, edition = {type = 'negative', negative = true}},
        [1] = {ability = {name = 'Joker', set = 'Joker', mult = 4}, save_fields = {center = 'j_joker'}, edition = {type = 'foil', foil = true, chips = 50}, sell_cost = 2, custom = {message = 'keep me'}},
        [2] = {ability = {name = 'Abstract Joker', set = 'Joker', extra = 3}, save_fields = {center = 'j_abstract'}},
        [3] = {ability = {name = 'Cavendish', set = 'Joker', extra = {Xmult = 3, odds = 1000}}, save_fields = {center = 'j_cavendish'}, edition = {type = 'holo', holo = true, mult = 10}},
        [4] = {ability = {name = 'DNA', set = 'Joker'}, save_fields = {center = 'j_dna'}, edition = {type = 'polychrome', polychrome = true, x_mult = 1.5}},
        [5] = {ability = {name = 'Joker', set = 'Joker', mult = 4}, save_fields = {center = 'j_joker'}},
      },
    },
    shop_jokers = {cards = {[1] = {ability = {name = 'Shop only', set = 'Joker'}}}},
    pack_cards = {cards = {[1] = {ability = {name = 'Pack only', set = 'Joker'}}}},
    consumeables = {config = {card_limit = 2}, cards = {}},
  },
  preserved = {[1] = 'numeric', ['1'] = 'string', [8] = 1.2345678901234567e100},
}
