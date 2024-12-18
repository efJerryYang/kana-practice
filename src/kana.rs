// Hiragana constants
pub const MAIN_HIRAGANA: [(&str, &str); 46] = [
    ("あ", "a"),  ("い", "i"),   ("う", "u"),   ("え", "e"),  ("お", "o"),
    ("か", "ka"), ("き", "ki"),  ("く", "ku"),  ("け", "ke"), ("こ", "ko"),
    ("さ", "sa"), ("し", "shi"), ("す", "su"),  ("せ", "se"), ("そ", "so"),
    ("た", "ta"), ("ち", "chi"), ("つ", "tsu"), ("て", "te"), ("と", "to"),
    ("な", "na"), ("に", "ni"),  ("ぬ", "nu"),  ("ね", "ne"), ("の", "no"),
    ("は", "ha"), ("ひ", "hi"),  ("ふ", "fu"),  ("へ", "he"), ("ほ", "ho"),
    ("ま", "ma"), ("み", "mi"),  ("む", "mu"),  ("め", "me"), ("も", "mo"),
    ("や", "ya"), ("ゆ", "yu"),  ("よ", "yo"),
    ("ら", "ra"), ("り", "ri"),  ("る", "ru"),  ("れ", "re"), ("ろ", "ro"),
    ("わ", "wa"), ("を", "wo"),  ("ん", "n"),
];

pub const DAKUTEN_HIRAGANA: [(&str, &str); 25] = [
    ("が", "ga"), ("ぎ", "gi"), ("ぐ", "gu"), ("げ", "ge"), ("ご", "go"),
    ("ざ", "za"), ("じ", "ji"), ("ず", "zu"), ("ぜ", "ze"), ("ぞ", "zo"),
    ("だ", "da"), ("ぢ", "di"), ("づ", "du"), ("で", "de"), ("ど", "do"),
    ("ば", "ba"), ("び", "bi"), ("ぶ", "bu"), ("べ", "be"), ("ぼ", "bo"),
    ("ぱ", "pa"), ("ぴ", "pi"), ("ぷ", "pu"), ("ぺ", "pe"), ("ぽ", "po"),
];

pub const COMBINATION_HIRAGANA: [(&str, &str); 33] = [
    ("きゃ", "kya"), ("きゅ", "kyu"), ("きょ", "kyo"),
    ("しゃ", "sha"), ("しゅ", "shu"), ("しょ", "sho"),
    ("ちゃ", "cha"), ("ちゅ", "chu"), ("ちょ", "cho"),
    ("にゃ", "nya"), ("にゅ", "nyu"), ("にょ", "nyo"),
    ("ひゃ", "hya"), ("ひゅ", "hyu"), ("ひょ", "hyo"),
    ("みゃ", "mya"), ("みゅ", "myu"), ("みょ", "myo"),
    ("りゃ", "rya"), ("りゅ", "ryu"), ("りょ", "ryo"),
    ("ぎゃ", "gya"), ("ぎゅ", "gyu"), ("ぎょ", "gyo"),
    ("じゃ", "ja"),  ("じゅ", "ju"),  ("じょ", "jo"),
    ("びゃ", "bya"), ("びゅ", "byu"), ("びょ", "byo"),
    ("ぴゃ", "pya"), ("ぴゅ", "pyu"), ("ぴょ", "pyo"),
];

// Katakana constants
pub const MAIN_KATAKANA: [(&str, &str); 46] = [
    ("ア", "a"),  ("イ", "i"),   ("ウ", "u"),   ("エ", "e"),  ("オ", "o"),
    ("カ", "ka"), ("キ", "ki"),  ("ク", "ku"),  ("ケ", "ke"), ("コ", "ko"),
    ("サ", "sa"), ("シ", "shi"), ("ス", "su"),  ("セ", "se"), ("ソ", "so"),
    ("タ", "ta"), ("チ", "chi"), ("ツ", "tsu"), ("テ", "te"), ("ト", "to"),
    ("ナ", "na"), ("ニ", "ni"),  ("ヌ", "nu"),  ("ネ", "ne"), ("ノ", "no"),
    ("ハ", "ha"), ("ヒ", "hi"),  ("フ", "fu"),  ("ヘ", "he"), ("ホ", "ho"),
    ("マ", "ma"), ("ミ", "mi"),  ("ム", "mu"),  ("メ", "me"), ("モ", "mo"),
    ("ヤ", "ya"), ("ユ", "yu"),  ("ヨ", "yo"),
    ("ラ", "ra"), ("リ", "ri"),  ("ル", "ru"),  ("レ", "re"), ("ロ", "ro"),
    ("ワ", "wa"), ("ヲ", "wo"),  ("ン", "n"),
];

pub const DAKUTEN_KATAKANA: [(&str, &str); 26] = [
    ("ガ", "ga"), ("ギ", "gi"), ("グ", "gu"), ("ゲ", "ge"), ("ゴ", "go"),
    ("ザ", "za"), ("ジ", "ji"), ("ズ", "zu"), ("ゼ", "ze"), ("ゾ", "zo"),
    ("ダ", "da"), ("ヂ", "ji"), ("ヅ", "zu"), ("デ", "de"), ("ド", "do"),
    ("バ", "ba"), ("ビ", "bi"), ("ブ", "bu"), ("ベ", "be"), ("ボ", "bo"),
    ("パ", "pa"), ("ピ", "pi"), ("プ", "pu"), ("ペ", "pe"), ("ポ", "po"),
    ("ヴ", "vu"),  // V-sound
];

pub const COMBINATION_KATAKANA: [(&str, &str); 55] = [
    // Standard y-combinations
    ("キャ", "kya"), ("キュ", "kyu"), ("キョ", "kyo"),
    ("シャ", "sha"), ("シュ", "shu"), ("ショ", "sho"),
    ("チャ", "cha"), ("チュ", "chu"), ("チョ", "cho"),
    ("ニャ", "nya"), ("ニュ", "nyu"), ("ニョ", "nyo"),
    ("ヒャ", "hya"), ("ヒュ", "hyu"), ("ヒョ", "hyo"),
    ("ミャ", "mya"), ("ミュ", "myu"), ("ミョ", "myo"),
    ("リャ", "rya"), ("リュ", "ryu"), ("リョ", "ryo"),
    ("ギャ", "gya"), ("ギュ", "gyu"), ("ギョ", "gyo"),
    ("ジャ", "ja"),  ("ジュ", "ju"),  ("ジョ", "jo"),
    ("ヂャ", "dya"), ("ヂュ", "dyu"), ("ヂョ", "dyo"),
    ("ビャ", "bya"), ("ビュ", "byu"), ("ビョ", "byo"),
    ("ピャ", "pya"), ("ピュ", "pyu"), ("ピョ", "pyo"),
    
    // Foreign sound combinations
    ("ヴァ", "va"), ("ヴィ", "vi"), ("ヴェ", "ve"), ("ヴォ", "vo"),
    ("ウィ", "wi"), ("ウェ", "we"), ("ウォ", "wo"),
    ("ファ", "fa"), ("フィ", "fi"), ("フェ", "fe"), ("フォ", "fo"),
    ("ツァ", "tsa"), ("ツィ", "tsi"), ("ツェ", "tse"), ("ツォ", "tso"),
    
    // Special combinations
    ("シェ", "she"),
    ("ジェ", "je"),
    ("チェ", "che"),
    
    // Irregular combinations from the image
    ("イェ", "ye")
];

const fn make_all_kana_by_type(
    main: [(&'static str, &'static str); 46],
    dakuten: [(&'static str, &'static str); 25],
    combination: [(&'static str, &'static str); 33],
) -> [(&'static str, &'static str); 104] {
    let mut result = [("", ""); 104];
    let mut offset = 0;
    let mut i = 0;

    // Copy main kana
    while i < 46 {
        result[offset + i] = main[i];
        i += 1;
    }
    offset += i;
    i = 0;

    // Copy dakuten kana
    while i < 25 {
        result[offset + i] = dakuten[i];
        i += 1;
    }
    offset += i;
    i = 0;

    // Copy combination kana
    while i < 33 {
        result[offset + i] = combination[i];
        i += 1;
    }

    result
}
const fn make_all_katakana() -> [(&'static str, &'static str); 127] {  // 46 + 26 + 55
    let mut result = [("", ""); 127];
    let mut offset = 0;
    let mut i = 0;

    // Copy main katakana
    while i < 46 {
        result[offset + i] = MAIN_KATAKANA[i];
        i += 1;
    }
    offset += i;
    i = 0;

    // Copy dakuten katakana
    while i < 26 {
        result[offset + i] = DAKUTEN_KATAKANA[i];
        i += 1;
    }
    offset += i;
    i = 0;

    // Copy combination katakana
    while i < 55 {
        result[offset + i] = COMBINATION_KATAKANA[i];
        i += 1;
    }

    result
}

pub const ALL_HIRAGANA: [(&str, &str); 104] = make_all_kana_by_type(MAIN_HIRAGANA, DAKUTEN_HIRAGANA, COMBINATION_HIRAGANA);
pub const ALL_KATAKANA: [(&str, &str); 127] = make_all_katakana();