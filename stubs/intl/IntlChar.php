<?php

class IntlChar
{
    public const UNICODE_VERSION = 13.0;
    public const CODEPOINT_MIN = 0;
    public const CODEPOINT_MAX = 1114111;
    public const FOLD_CASE_DEFAULT = 0;
    public const FOLD_CASE_EXCLUDE_SPECIAL_I = 1;
    public const PROPERTY_ALPHABETIC = 0;
    public const PROPERTY_BINARY_START = 0;
    public const PROPERTY_ASCII_HEX_DIGIT = 1;
    public const PROPERTY_BIDI_CONTROL = 2;
    public const PROPERTY_BIDI_MIRRORED = 3;
    public const PROPERTY_DASH = 4;
    public const PROPERTY_DEFAULT_IGNORABLE_CODE_POINT = 5;
    public const PROPERTY_DEPRECATED = 6;
    public const PROPERTY_DIACRITIC = 7;
    public const PROPERTY_EXTENDER = 8;
    public const PROPERTY_FULL_COMPOSITION_EXCLUSION = 9;
    public const PROPERTY_GRAPHEME_BASE = 10;
    public const PROPERTY_GRAPHEME_EXTEND = 11;
    public const PROPERTY_GRAPHEME_LINK = 12;
    public const PROPERTY_HEX_DIGIT = 13;
    public const PROPERTY_HYPHEN = 14;
    public const PROPERTY_ID_CONTINUE = 15;
    public const PROPERTY_ID_START = 16;
    public const PROPERTY_IDEOGRAPHIC = 17;
    public const PROPERTY_IDS_BINARY_OPERATOR = 18;
    public const PROPERTY_IDS_TRINARY_OPERATOR = 19;
    public const PROPERTY_JOIN_CONTROL = 20;
    public const PROPERTY_LOGICAL_ORDER_EXCEPTION = 21;
    public const PROPERTY_LOWERCASE = 22;
    public const PROPERTY_MATH = 23;
    public const PROPERTY_NONCHARACTER_CODE_POINT = 24;
    public const PROPERTY_QUOTATION_MARK = 25;
    public const PROPERTY_RADICAL = 26;
    public const PROPERTY_SOFT_DOTTED = 27;
    public const PROPERTY_TERMINAL_PUNCTUATION = 28;
    public const PROPERTY_UNIFIED_IDEOGRAPH = 29;
    public const PROPERTY_UPPERCASE = 30;
    public const PROPERTY_WHITE_SPACE = 31;
    public const PROPERTY_XID_CONTINUE = 32;
    public const PROPERTY_XID_START = 33;
    public const PROPERTY_CASE_SENSITIVE = 34;
    public const PROPERTY_S_TERM = 35;
    public const PROPERTY_VARIATION_SELECTOR = 36;
    public const PROPERTY_NFD_INERT = 37;
    public const PROPERTY_NFKD_INERT = 38;
    public const PROPERTY_NFC_INERT = 39;
    public const PROPERTY_NFKC_INERT = 40;
    public const PROPERTY_SEGMENT_STARTER = 41;
    public const PROPERTY_PATTERN_SYNTAX = 42;
    public const PROPERTY_PATTERN_WHITE_SPACE = 43;
    public const PROPERTY_POSIX_ALNUM = 44;
    public const PROPERTY_POSIX_BLANK = 45;
    public const PROPERTY_POSIX_GRAPH = 46;
    public const PROPERTY_POSIX_PRINT = 47;
    public const PROPERTY_POSIX_XDIGIT = 48;
    public const PROPERTY_CASED = 49;
    public const PROPERTY_CASE_IGNORABLE = 50;
    public const PROPERTY_CHANGES_WHEN_LOWERCASED = 51;
    public const PROPERTY_CHANGES_WHEN_UPPERCASED = 52;
    public const PROPERTY_CHANGES_WHEN_TITLECASED = 53;
    public const PROPERTY_CHANGES_WHEN_CASEFOLDED = 54;
    public const PROPERTY_CHANGES_WHEN_CASEMAPPED = 55;
    public const PROPERTY_CHANGES_WHEN_NFKC_CASEFOLDED = 56;
    public const PROPERTY_BINARY_LIMIT = 65;
    public const PROPERTY_BIDI_CLASS = 4096;
    public const PROPERTY_INT_START = 4096;
    public const PROPERTY_BLOCK = 4097;
    public const PROPERTY_CANONICAL_COMBINING_CLASS = 4098;
    public const PROPERTY_DECOMPOSITION_TYPE = 4099;
    public const PROPERTY_EAST_ASIAN_WIDTH = 4100;
    public const PROPERTY_GENERAL_CATEGORY = 4101;
    public const PROPERTY_JOINING_GROUP = 4102;
    public const PROPERTY_JOINING_TYPE = 4103;
    public const PROPERTY_LINE_BREAK = 4104;
    public const PROPERTY_NUMERIC_TYPE = 4105;
    public const PROPERTY_SCRIPT = 4106;
    public const PROPERTY_HANGUL_SYLLABLE_TYPE = 4107;
    public const PROPERTY_NFD_QUICK_CHECK = 4108;
    public const PROPERTY_NFKD_QUICK_CHECK = 4109;
    public const PROPERTY_NFC_QUICK_CHECK = 4110;
    public const PROPERTY_NFKC_QUICK_CHECK = 4111;
    public const PROPERTY_LEAD_CANONICAL_COMBINING_CLASS = 4112;
    public const PROPERTY_TRAIL_CANONICAL_COMBINING_CLASS = 4113;
    public const PROPERTY_GRAPHEME_CLUSTER_BREAK = 4114;
    public const PROPERTY_SENTENCE_BREAK = 4115;
    public const PROPERTY_WORD_BREAK = 4116;
    public const PROPERTY_BIDI_PAIRED_BRACKET_TYPE = 4117;
    public const PROPERTY_INT_LIMIT = 4121;
    public const PROPERTY_GENERAL_CATEGORY_MASK = 8192;
    public const PROPERTY_MASK_START = 8192;
    public const PROPERTY_MASK_LIMIT = 8193;
    public const PROPERTY_NUMERIC_VALUE = 12288;
    public const PROPERTY_DOUBLE_START = 12288;
    public const PROPERTY_DOUBLE_LIMIT = 12289;
    public const PROPERTY_AGE = 16384;
    public const PROPERTY_STRING_START = 16384;
    public const PROPERTY_BIDI_MIRRORING_GLYPH = 16385;
    public const PROPERTY_CASE_FOLDING = 16386;
    public const PROPERTY_ISO_COMMENT = 16387;
    public const PROPERTY_LOWERCASE_MAPPING = 16388;
    public const PROPERTY_NAME = 16389;
    public const PROPERTY_SIMPLE_CASE_FOLDING = 16390;
    public const PROPERTY_SIMPLE_LOWERCASE_MAPPING = 16391;
    public const PROPERTY_SIMPLE_TITLECASE_MAPPING = 16392;
    public const PROPERTY_SIMPLE_UPPERCASE_MAPPING = 16393;
    public const PROPERTY_TITLECASE_MAPPING = 16394;
    public const PROPERTY_UNICODE_1_NAME = 16395;
    public const PROPERTY_UPPERCASE_MAPPING = 16396;
    public const PROPERTY_BIDI_PAIRED_BRACKET = 16397;
    public const PROPERTY_STRING_LIMIT = 16398;
    public const PROPERTY_SCRIPT_EXTENSIONS = 28672;
    public const PROPERTY_OTHER_PROPERTY_START = 28672;
    public const PROPERTY_OTHER_PROPERTY_LIMIT = 28673;
    public const PROPERTY_INVALID_CODE = -1;
    public const CHAR_CATEGORY_UNASSIGNED = 0;
    public const CHAR_CATEGORY_GENERAL_OTHER_TYPES = 0;
    public const CHAR_CATEGORY_UPPERCASE_LETTER = 1;
    public const CHAR_CATEGORY_LOWERCASE_LETTER = 2;
    public const CHAR_CATEGORY_TITLECASE_LETTER = 3;
    public const CHAR_CATEGORY_MODIFIER_LETTER = 4;
    public const CHAR_CATEGORY_OTHER_LETTER = 5;
    public const CHAR_CATEGORY_NON_SPACING_MARK = 6;
    public const CHAR_CATEGORY_ENCLOSING_MARK = 7;
    public const CHAR_CATEGORY_COMBINING_SPACING_MARK = 8;
    public const CHAR_CATEGORY_DECIMAL_DIGIT_NUMBER = 9;
    public const CHAR_CATEGORY_LETTER_NUMBER = 10;
    public const CHAR_CATEGORY_OTHER_NUMBER = 11;
    public const CHAR_CATEGORY_SPACE_SEPARATOR = 12;
    public const CHAR_CATEGORY_LINE_SEPARATOR = 13;
    public const CHAR_CATEGORY_PARAGRAPH_SEPARATOR = 14;
    public const CHAR_CATEGORY_CONTROL_CHAR = 15;
    public const CHAR_CATEGORY_FORMAT_CHAR = 16;
    public const CHAR_CATEGORY_PRIVATE_USE_CHAR = 17;
    public const CHAR_CATEGORY_SURROGATE = 18;
    public const CHAR_CATEGORY_DASH_PUNCTUATION = 19;
    public const CHAR_CATEGORY_START_PUNCTUATION = 20;
    public const CHAR_CATEGORY_END_PUNCTUATION = 21;
    public const CHAR_CATEGORY_CONNECTOR_PUNCTUATION = 22;
    public const CHAR_CATEGORY_OTHER_PUNCTUATION = 23;
    public const CHAR_CATEGORY_MATH_SYMBOL = 24;
    public const CHAR_CATEGORY_CURRENCY_SYMBOL = 25;
    public const CHAR_CATEGORY_MODIFIER_SYMBOL = 26;
    public const CHAR_CATEGORY_OTHER_SYMBOL = 27;
    public const CHAR_CATEGORY_INITIAL_PUNCTUATION = 28;
    public const CHAR_CATEGORY_FINAL_PUNCTUATION = 29;
    public const CHAR_CATEGORY_CHAR_CATEGORY_COUNT = 30;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT = 0;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT = 1;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER = 2;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER_SEPARATOR = 3;
    public const CHAR_DIRECTION_EUROPEAN_NUMBER_TERMINATOR = 4;
    public const CHAR_DIRECTION_ARABIC_NUMBER = 5;
    public const CHAR_DIRECTION_COMMON_NUMBER_SEPARATOR = 6;
    public const CHAR_DIRECTION_BLOCK_SEPARATOR = 7;
    public const CHAR_DIRECTION_SEGMENT_SEPARATOR = 8;
    public const CHAR_DIRECTION_WHITE_SPACE_NEUTRAL = 9;
    public const CHAR_DIRECTION_OTHER_NEUTRAL = 10;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_EMBEDDING = 11;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_OVERRIDE = 12;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_ARABIC = 13;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_EMBEDDING = 14;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_OVERRIDE = 15;
    public const CHAR_DIRECTION_POP_DIRECTIONAL_FORMAT = 16;
    public const CHAR_DIRECTION_DIR_NON_SPACING_MARK = 17;
    public const CHAR_DIRECTION_BOUNDARY_NEUTRAL = 18;
    public const CHAR_DIRECTION_FIRST_STRONG_ISOLATE = 19;
    public const CHAR_DIRECTION_LEFT_TO_RIGHT_ISOLATE = 20;
    public const CHAR_DIRECTION_RIGHT_TO_LEFT_ISOLATE = 21;
    public const CHAR_DIRECTION_POP_DIRECTIONAL_ISOLATE = 22;
    public const CHAR_DIRECTION_CHAR_DIRECTION_COUNT = 23;
    public const BLOCK_CODE_NO_BLOCK = 0;
    public const BLOCK_CODE_BASIC_LATIN = 1;
    public const BLOCK_CODE_LATIN_1_SUPPLEMENT = 2;
    public const BLOCK_CODE_LATIN_EXTENDED_A = 3;
    public const BLOCK_CODE_LATIN_EXTENDED_B = 4;
    public const BLOCK_CODE_IPA_EXTENSIONS = 5;
    public const BLOCK_CODE_SPACING_MODIFIER_LETTERS = 6;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS = 7;
    public const BLOCK_CODE_GREEK = 8;
    public const BLOCK_CODE_CYRILLIC = 9;
    public const BLOCK_CODE_ARMENIAN = 10;
    public const BLOCK_CODE_HEBREW = 11;
    public const BLOCK_CODE_ARABIC = 12;
    public const BLOCK_CODE_SYRIAC = 13;
    public const BLOCK_CODE_THAANA = 14;
    public const BLOCK_CODE_DEVANAGARI = 15;
    public const BLOCK_CODE_BENGALI = 16;
    public const BLOCK_CODE_GURMUKHI = 17;
    public const BLOCK_CODE_GUJARATI = 18;
    public const BLOCK_CODE_ORIYA = 19;
    public const BLOCK_CODE_TAMIL = 20;
    public const BLOCK_CODE_TELUGU = 21;
    public const BLOCK_CODE_KANNADA = 22;
    public const BLOCK_CODE_MALAYALAM = 23;
    public const BLOCK_CODE_SINHALA = 24;
    public const BLOCK_CODE_THAI = 25;
    public const BLOCK_CODE_LAO = 26;
    public const BLOCK_CODE_TIBETAN = 27;
    public const BLOCK_CODE_MYANMAR = 28;
    public const BLOCK_CODE_GEORGIAN = 29;
    public const BLOCK_CODE_HANGUL_JAMO = 30;
    public const BLOCK_CODE_ETHIOPIC = 31;
    public const BLOCK_CODE_CHEROKEE = 32;
    public const BLOCK_CODE_UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS = 33;
    public const BLOCK_CODE_OGHAM = 34;
    public const BLOCK_CODE_RUNIC = 35;
    public const BLOCK_CODE_KHMER = 36;
    public const BLOCK_CODE_MONGOLIAN = 37;
    public const BLOCK_CODE_LATIN_EXTENDED_ADDITIONAL = 38;
    public const BLOCK_CODE_GREEK_EXTENDED = 39;
    public const BLOCK_CODE_GENERAL_PUNCTUATION = 40;
    public const BLOCK_CODE_SUPERSCRIPTS_AND_SUBSCRIPTS = 41;
    public const BLOCK_CODE_CURRENCY_SYMBOLS = 42;
    public const BLOCK_CODE_COMBINING_MARKS_FOR_SYMBOLS = 43;
    public const BLOCK_CODE_LETTERLIKE_SYMBOLS = 44;
    public const BLOCK_CODE_NUMBER_FORMS = 45;
    public const BLOCK_CODE_ARROWS = 46;
    public const BLOCK_CODE_MATHEMATICAL_OPERATORS = 47;
    public const BLOCK_CODE_MISCELLANEOUS_TECHNICAL = 48;
    public const BLOCK_CODE_CONTROL_PICTURES = 49;
    public const BLOCK_CODE_OPTICAL_CHARACTER_RECOGNITION = 50;
    public const BLOCK_CODE_ENCLOSED_ALPHANUMERICS = 51;
    public const BLOCK_CODE_BOX_DRAWING = 52;
    public const BLOCK_CODE_BLOCK_ELEMENTS = 53;
    public const BLOCK_CODE_GEOMETRIC_SHAPES = 54;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS = 55;
    public const BLOCK_CODE_DINGBATS = 56;
    public const BLOCK_CODE_BRAILLE_PATTERNS = 57;
    public const BLOCK_CODE_CJK_RADICALS_SUPPLEMENT = 58;
    public const BLOCK_CODE_KANGXI_RADICALS = 59;
    public const BLOCK_CODE_IDEOGRAPHIC_DESCRIPTION_CHARACTERS = 60;
    public const BLOCK_CODE_CJK_SYMBOLS_AND_PUNCTUATION = 61;
    public const BLOCK_CODE_HIRAGANA = 62;
    public const BLOCK_CODE_KATAKANA = 63;
    public const BLOCK_CODE_BOPOMOFO = 64;
    public const BLOCK_CODE_HANGUL_COMPATIBILITY_JAMO = 65;
    public const BLOCK_CODE_KANBUN = 66;
    public const BLOCK_CODE_BOPOMOFO_EXTENDED = 67;
    public const BLOCK_CODE_ENCLOSED_CJK_LETTERS_AND_MONTHS = 68;
    public const BLOCK_CODE_CJK_COMPATIBILITY = 69;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A = 70;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS = 71;
    public const BLOCK_CODE_YI_SYLLABLES = 72;
    public const BLOCK_CODE_YI_RADICALS = 73;
    public const BLOCK_CODE_HANGUL_SYLLABLES = 74;
    public const BLOCK_CODE_HIGH_SURROGATES = 75;
    public const BLOCK_CODE_HIGH_PRIVATE_USE_SURROGATES = 76;
    public const BLOCK_CODE_LOW_SURROGATES = 77;
    public const BLOCK_CODE_PRIVATE_USE_AREA = 78;
    public const BLOCK_CODE_PRIVATE_USE = 78;
    public const BLOCK_CODE_CJK_COMPATIBILITY_IDEOGRAPHS = 79;
    public const BLOCK_CODE_ALPHABETIC_PRESENTATION_FORMS = 80;
    public const BLOCK_CODE_ARABIC_PRESENTATION_FORMS_A = 81;
    public const BLOCK_CODE_COMBINING_HALF_MARKS = 82;
    public const BLOCK_CODE_CJK_COMPATIBILITY_FORMS = 83;
    public const BLOCK_CODE_SMALL_FORM_VARIANTS = 84;
    public const BLOCK_CODE_ARABIC_PRESENTATION_FORMS_B = 85;
    public const BLOCK_CODE_SPECIALS = 86;
    public const BLOCK_CODE_HALFWIDTH_AND_FULLWIDTH_FORMS = 87;
    public const BLOCK_CODE_OLD_ITALIC = 88;
    public const BLOCK_CODE_GOTHIC = 89;
    public const BLOCK_CODE_DESERET = 90;
    public const BLOCK_CODE_BYZANTINE_MUSICAL_SYMBOLS = 91;
    public const BLOCK_CODE_MUSICAL_SYMBOLS = 92;
    public const BLOCK_CODE_MATHEMATICAL_ALPHANUMERIC_SYMBOLS = 93;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B = 94;
    public const BLOCK_CODE_CJK_COMPATIBILITY_IDEOGRAPHS_SUPPLEMENT = 95;
    public const BLOCK_CODE_TAGS = 96;
    public const BLOCK_CODE_CYRILLIC_SUPPLEMENT = 97;
    public const BLOCK_CODE_CYRILLIC_SUPPLEMENTARY = 97;
    public const BLOCK_CODE_TAGALOG = 98;
    public const BLOCK_CODE_HANUNOO = 99;
    public const BLOCK_CODE_BUHID = 100;
    public const BLOCK_CODE_TAGBANWA = 101;
    public const BLOCK_CODE_MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A = 102;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_A = 103;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_B = 104;
    public const BLOCK_CODE_MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B = 105;
    public const BLOCK_CODE_SUPPLEMENTAL_MATHEMATICAL_OPERATORS = 106;
    public const BLOCK_CODE_KATAKANA_PHONETIC_EXTENSIONS = 107;
    public const BLOCK_CODE_VARIATION_SELECTORS = 108;
    public const BLOCK_CODE_SUPPLEMENTARY_PRIVATE_USE_AREA_A = 109;
    public const BLOCK_CODE_SUPPLEMENTARY_PRIVATE_USE_AREA_B = 110;
    public const BLOCK_CODE_LIMBU = 111;
    public const BLOCK_CODE_TAI_LE = 112;
    public const BLOCK_CODE_KHMER_SYMBOLS = 113;
    public const BLOCK_CODE_PHONETIC_EXTENSIONS = 114;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS_AND_ARROWS = 115;
    public const BLOCK_CODE_YIJING_HEXAGRAM_SYMBOLS = 116;
    public const BLOCK_CODE_LINEAR_B_SYLLABARY = 117;
    public const BLOCK_CODE_LINEAR_B_IDEOGRAMS = 118;
    public const BLOCK_CODE_AEGEAN_NUMBERS = 119;
    public const BLOCK_CODE_UGARITIC = 120;
    public const BLOCK_CODE_SHAVIAN = 121;
    public const BLOCK_CODE_OSMANYA = 122;
    public const BLOCK_CODE_CYPRIOT_SYLLABARY = 123;
    public const BLOCK_CODE_TAI_XUAN_JING_SYMBOLS = 124;
    public const BLOCK_CODE_VARIATION_SELECTORS_SUPPLEMENT = 125;
    public const BLOCK_CODE_ANCIENT_GREEK_MUSICAL_NOTATION = 126;
    public const BLOCK_CODE_ANCIENT_GREEK_NUMBERS = 127;
    public const BLOCK_CODE_ARABIC_SUPPLEMENT = 128;
    public const BLOCK_CODE_BUGINESE = 129;
    public const BLOCK_CODE_CJK_STROKES = 130;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS_SUPPLEMENT = 131;
    public const BLOCK_CODE_COPTIC = 132;
    public const BLOCK_CODE_ETHIOPIC_EXTENDED = 133;
    public const BLOCK_CODE_ETHIOPIC_SUPPLEMENT = 134;
    public const BLOCK_CODE_GEORGIAN_SUPPLEMENT = 135;
    public const BLOCK_CODE_GLAGOLITIC = 136;
    public const BLOCK_CODE_KHAROSHTHI = 137;
    public const BLOCK_CODE_MODIFIER_TONE_LETTERS = 138;
    public const BLOCK_CODE_NEW_TAI_LUE = 139;
    public const BLOCK_CODE_OLD_PERSIAN = 140;
    public const BLOCK_CODE_PHONETIC_EXTENSIONS_SUPPLEMENT = 141;
    public const BLOCK_CODE_SUPPLEMENTAL_PUNCTUATION = 142;
    public const BLOCK_CODE_SYLOTI_NAGRI = 143;
    public const BLOCK_CODE_TIFINAGH = 144;
    public const BLOCK_CODE_VERTICAL_FORMS = 145;
    public const BLOCK_CODE_NKO = 146;
    public const BLOCK_CODE_BALINESE = 147;
    public const BLOCK_CODE_LATIN_EXTENDED_C = 148;
    public const BLOCK_CODE_LATIN_EXTENDED_D = 149;
    public const BLOCK_CODE_PHAGS_PA = 150;
    public const BLOCK_CODE_PHOENICIAN = 151;
    public const BLOCK_CODE_CUNEIFORM = 152;
    public const BLOCK_CODE_CUNEIFORM_NUMBERS_AND_PUNCTUATION = 153;
    public const BLOCK_CODE_COUNTING_ROD_NUMERALS = 154;
    public const BLOCK_CODE_SUNDANESE = 155;
    public const BLOCK_CODE_LEPCHA = 156;
    public const BLOCK_CODE_OL_CHIKI = 157;
    public const BLOCK_CODE_CYRILLIC_EXTENDED_A = 158;
    public const BLOCK_CODE_VAI = 159;
    public const BLOCK_CODE_CYRILLIC_EXTENDED_B = 160;
    public const BLOCK_CODE_SAURASHTRA = 161;
    public const BLOCK_CODE_KAYAH_LI = 162;
    public const BLOCK_CODE_REJANG = 163;
    public const BLOCK_CODE_CHAM = 164;
    public const BLOCK_CODE_ANCIENT_SYMBOLS = 165;
    public const BLOCK_CODE_PHAISTOS_DISC = 166;
    public const BLOCK_CODE_LYCIAN = 167;
    public const BLOCK_CODE_CARIAN = 168;
    public const BLOCK_CODE_LYDIAN = 169;
    public const BLOCK_CODE_MAHJONG_TILES = 170;
    public const BLOCK_CODE_DOMINO_TILES = 171;
    public const BLOCK_CODE_SAMARITAN = 172;
    public const BLOCK_CODE_UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED = 173;
    public const BLOCK_CODE_TAI_THAM = 174;
    public const BLOCK_CODE_VEDIC_EXTENSIONS = 175;
    public const BLOCK_CODE_LISU = 176;
    public const BLOCK_CODE_BAMUM = 177;
    public const BLOCK_CODE_COMMON_INDIC_NUMBER_FORMS = 178;
    public const BLOCK_CODE_DEVANAGARI_EXTENDED = 179;
    public const BLOCK_CODE_HANGUL_JAMO_EXTENDED_A = 180;
    public const BLOCK_CODE_JAVANESE = 181;
    public const BLOCK_CODE_MYANMAR_EXTENDED_A = 182;
    public const BLOCK_CODE_TAI_VIET = 183;
    public const BLOCK_CODE_MEETEI_MAYEK = 184;
    public const BLOCK_CODE_HANGUL_JAMO_EXTENDED_B = 185;
    public const BLOCK_CODE_IMPERIAL_ARAMAIC = 186;
    public const BLOCK_CODE_OLD_SOUTH_ARABIAN = 187;
    public const BLOCK_CODE_AVESTAN = 188;
    public const BLOCK_CODE_INSCRIPTIONAL_PARTHIAN = 189;
    public const BLOCK_CODE_INSCRIPTIONAL_PAHLAVI = 190;
    public const BLOCK_CODE_OLD_TURKIC = 191;
    public const BLOCK_CODE_RUMI_NUMERAL_SYMBOLS = 192;
    public const BLOCK_CODE_KAITHI = 193;
    public const BLOCK_CODE_EGYPTIAN_HIEROGLYPHS = 194;
    public const BLOCK_CODE_ENCLOSED_ALPHANUMERIC_SUPPLEMENT = 195;
    public const BLOCK_CODE_ENCLOSED_IDEOGRAPHIC_SUPPLEMENT = 196;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C = 197;
    public const BLOCK_CODE_MANDAIC = 198;
    public const BLOCK_CODE_BATAK = 199;
    public const BLOCK_CODE_ETHIOPIC_EXTENDED_A = 200;
    public const BLOCK_CODE_BRAHMI = 201;
    public const BLOCK_CODE_BAMUM_SUPPLEMENT = 202;
    public const BLOCK_CODE_KANA_SUPPLEMENT = 203;
    public const BLOCK_CODE_PLAYING_CARDS = 204;
    public const BLOCK_CODE_MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS = 205;
    public const BLOCK_CODE_EMOTICONS = 206;
    public const BLOCK_CODE_TRANSPORT_AND_MAP_SYMBOLS = 207;
    public const BLOCK_CODE_ALCHEMICAL_SYMBOLS = 208;
    public const BLOCK_CODE_CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D = 209;
    public const BLOCK_CODE_ARABIC_EXTENDED_A = 210;
    public const BLOCK_CODE_ARABIC_MATHEMATICAL_ALPHABETIC_SYMBOLS = 211;
    public const BLOCK_CODE_CHAKMA = 212;
    public const BLOCK_CODE_MEETEI_MAYEK_EXTENSIONS = 213;
    public const BLOCK_CODE_MEROITIC_CURSIVE = 214;
    public const BLOCK_CODE_MEROITIC_HIEROGLYPHS = 215;
    public const BLOCK_CODE_MIAO = 216;
    public const BLOCK_CODE_SHARADA = 217;
    public const BLOCK_CODE_SORA_SOMPENG = 218;
    public const BLOCK_CODE_SUNDANESE_SUPPLEMENT = 219;
    public const BLOCK_CODE_TAKRI = 220;
    public const BLOCK_CODE_BASSA_VAH = 221;
    public const BLOCK_CODE_CAUCASIAN_ALBANIAN = 222;
    public const BLOCK_CODE_COPTIC_EPACT_NUMBERS = 223;
    public const BLOCK_CODE_COMBINING_DIACRITICAL_MARKS_EXTENDED = 224;
    public const BLOCK_CODE_DUPLOYAN = 225;
    public const BLOCK_CODE_ELBASAN = 226;
    public const BLOCK_CODE_GEOMETRIC_SHAPES_EXTENDED = 227;
    public const BLOCK_CODE_GRANTHA = 228;
    public const BLOCK_CODE_KHOJKI = 229;
    public const BLOCK_CODE_KHUDAWADI = 230;
    public const BLOCK_CODE_LATIN_EXTENDED_E = 231;
    public const BLOCK_CODE_LINEAR_A = 232;
    public const BLOCK_CODE_MAHAJANI = 233;
    public const BLOCK_CODE_MANICHAEAN = 234;
    public const BLOCK_CODE_MENDE_KIKAKUI = 235;
    public const BLOCK_CODE_MODI = 236;
    public const BLOCK_CODE_MRO = 237;
    public const BLOCK_CODE_MYANMAR_EXTENDED_B = 238;
    public const BLOCK_CODE_NABATAEAN = 239;
    public const BLOCK_CODE_OLD_NORTH_ARABIAN = 240;
    public const BLOCK_CODE_OLD_PERMIC = 241;
    public const BLOCK_CODE_ORNAMENTAL_DINGBATS = 242;
    public const BLOCK_CODE_PAHAWH_HMONG = 243;
    public const BLOCK_CODE_PALMYRENE = 244;
    public const BLOCK_CODE_PAU_CIN_HAU = 245;
    public const BLOCK_CODE_PSALTER_PAHLAVI = 246;
    public const BLOCK_CODE_SHORTHAND_FORMAT_CONTROLS = 247;
    public const BLOCK_CODE_SIDDHAM = 248;
    public const BLOCK_CODE_SINHALA_ARCHAIC_NUMBERS = 249;
    public const BLOCK_CODE_SUPPLEMENTAL_ARROWS_C = 250;
    public const BLOCK_CODE_TIRHUTA = 251;
    public const BLOCK_CODE_WARANG_CITI = 252;
    public const BLOCK_CODE_COUNT = 309;
    public const BLOCK_CODE_INVALID_CODE = -1;
    public const BPT_NONE = 0;
    public const BPT_OPEN = 1;
    public const BPT_CLOSE = 2;
    public const BPT_COUNT = 3;
    public const EA_NEUTRAL = 0;
    public const EA_AMBIGUOUS = 1;
    public const EA_HALFWIDTH = 2;
    public const EA_FULLWIDTH = 3;
    public const EA_NARROW = 4;
    public const EA_WIDE = 5;
    public const EA_COUNT = 6;
    public const UNICODE_CHAR_NAME = 0;
    public const UNICODE_10_CHAR_NAME = 1;
    public const EXTENDED_CHAR_NAME = 2;
    public const CHAR_NAME_ALIAS = 3;
    public const CHAR_NAME_CHOICE_COUNT = 4;
    public const SHORT_PROPERTY_NAME = 0;
    public const LONG_PROPERTY_NAME = 1;
    public const PROPERTY_NAME_CHOICE_COUNT = 2;
    public const DT_NONE = 0;
    public const DT_CANONICAL = 1;
    public const DT_COMPAT = 2;
    public const DT_CIRCLE = 3;
    public const DT_FINAL = 4;
    public const DT_FONT = 5;
    public const DT_FRACTION = 6;
    public const DT_INITIAL = 7;
    public const DT_ISOLATED = 8;
    public const DT_MEDIAL = 9;
    public const DT_NARROW = 10;
    public const DT_NOBREAK = 11;
    public const DT_SMALL = 12;
    public const DT_SQUARE = 13;
    public const DT_SUB = 14;
    public const DT_SUPER = 15;
    public const DT_VERTICAL = 16;
    public const DT_WIDE = 17;
    public const DT_COUNT = 18;
    public const JT_NON_JOINING = 0;
    public const JT_JOIN_CAUSING = 1;
    public const JT_DUAL_JOINING = 2;
    public const JT_LEFT_JOINING = 3;
    public const JT_RIGHT_JOINING = 4;
    public const JT_TRANSPARENT = 5;
    public const JT_COUNT = 6;
    public const JG_NO_JOINING_GROUP = 0;
    public const JG_AIN = 1;
    public const JG_ALAPH = 2;
    public const JG_ALEF = 3;
    public const JG_BEH = 4;
    public const JG_BETH = 5;
    public const JG_DAL = 6;
    public const JG_DALATH_RISH = 7;
    public const JG_E = 8;
    public const JG_FEH = 9;
    public const JG_FINAL_SEMKATH = 10;
    public const JG_GAF = 11;
    public const JG_GAMAL = 12;
    public const JG_HAH = 13;
    public const JG_TEH_MARBUTA_GOAL = 14;
    public const JG_HAMZA_ON_HEH_GOAL = 14;
    public const JG_HE = 15;
    public const JG_HEH = 16;
    public const JG_HEH_GOAL = 17;
    public const JG_HETH = 18;
    public const JG_KAF = 19;
    public const JG_KAPH = 20;
    public const JG_KNOTTED_HEH = 21;
    public const JG_LAM = 22;
    public const JG_LAMADH = 23;
    public const JG_MEEM = 24;
    public const JG_MIM = 25;
    public const JG_NOON = 26;
    public const JG_NUN = 27;
    public const JG_PE = 28;
    public const JG_QAF = 29;
    public const JG_QAPH = 30;
    public const JG_REH = 31;
    public const JG_REVERSED_PE = 32;
    public const JG_SAD = 33;
    public const JG_SADHE = 34;
    public const JG_SEEN = 35;
    public const JG_SEMKATH = 36;
    public const JG_SHIN = 37;
    public const JG_SWASH_KAF = 38;
    public const JG_SYRIAC_WAW = 39;
    public const JG_TAH = 40;
    public const JG_TAW = 41;
    public const JG_TEH_MARBUTA = 42;
    public const JG_TETH = 43;
    public const JG_WAW = 44;
    public const JG_YEH = 45;
    public const JG_YEH_BARREE = 46;
    public const JG_YEH_WITH_TAIL = 47;
    public const JG_YUDH = 48;
    public const JG_YUDH_HE = 49;
    public const JG_ZAIN = 50;
    public const JG_FE = 51;
    public const JG_KHAPH = 52;
    public const JG_ZHAIN = 53;
    public const JG_BURUSHASKI_YEH_BARREE = 54;
    public const JG_FARSI_YEH = 55;
    public const JG_NYA = 56;
    public const JG_ROHINGYA_YEH = 57;
    public const JG_MANICHAEAN_ALEPH = 58;
    public const JG_MANICHAEAN_AYIN = 59;
    public const JG_MANICHAEAN_BETH = 60;
    public const JG_MANICHAEAN_DALETH = 61;
    public const JG_MANICHAEAN_DHAMEDH = 62;
    public const JG_MANICHAEAN_FIVE = 63;
    public const JG_MANICHAEAN_GIMEL = 64;
    public const JG_MANICHAEAN_HETH = 65;
    public const JG_MANICHAEAN_HUNDRED = 66;
    public const JG_MANICHAEAN_KAPH = 67;
    public const JG_MANICHAEAN_LAMEDH = 68;
    public const JG_MANICHAEAN_MEM = 69;
    public const JG_MANICHAEAN_NUN = 70;
    public const JG_MANICHAEAN_ONE = 71;
    public const JG_MANICHAEAN_PE = 72;
    public const JG_MANICHAEAN_QOPH = 73;
    public const JG_MANICHAEAN_RESH = 74;
    public const JG_MANICHAEAN_SADHE = 75;
    public const JG_MANICHAEAN_SAMEKH = 76;
    public const JG_MANICHAEAN_TAW = 77;
    public const JG_MANICHAEAN_TEN = 78;
    public const JG_MANICHAEAN_TETH = 79;
    public const JG_MANICHAEAN_THAMEDH = 80;
    public const JG_MANICHAEAN_TWENTY = 81;
    public const JG_MANICHAEAN_WAW = 82;
    public const JG_MANICHAEAN_YODH = 83;
    public const JG_MANICHAEAN_ZAYIN = 84;
    public const JG_STRAIGHT_WAW = 85;
    public const JG_COUNT = 102;
    public const GCB_OTHER = 0;
    public const GCB_CONTROL = 1;
    public const GCB_CR = 2;
    public const GCB_EXTEND = 3;
    public const GCB_L = 4;
    public const GCB_LF = 5;
    public const GCB_LV = 6;
    public const GCB_LVT = 7;
    public const GCB_T = 8;
    public const GCB_V = 9;
    public const GCB_SPACING_MARK = 10;
    public const GCB_PREPEND = 11;
    public const GCB_REGIONAL_INDICATOR = 12;
    public const GCB_COUNT = 18;
    public const WB_OTHER = 0;
    public const WB_ALETTER = 1;
    public const WB_FORMAT = 2;
    public const WB_KATAKANA = 3;
    public const WB_MIDLETTER = 4;
    public const WB_MIDNUM = 5;
    public const WB_NUMERIC = 6;
    public const WB_EXTENDNUMLET = 7;
    public const WB_CR = 8;
    public const WB_EXTEND = 9;
    public const WB_LF = 10;
    public const WB_MIDNUMLET = 11;
    public const WB_NEWLINE = 12;
    public const WB_REGIONAL_INDICATOR = 13;
    public const WB_HEBREW_LETTER = 14;
    public const WB_SINGLE_QUOTE = 15;
    public const WB_DOUBLE_QUOTE = 16;
    public const WB_COUNT = 23;
    public const SB_OTHER = 0;
    public const SB_ATERM = 1;
    public const SB_CLOSE = 2;
    public const SB_FORMAT = 3;
    public const SB_LOWER = 4;
    public const SB_NUMERIC = 5;
    public const SB_OLETTER = 6;
    public const SB_SEP = 7;
    public const SB_SP = 8;
    public const SB_STERM = 9;
    public const SB_UPPER = 10;
    public const SB_CR = 11;
    public const SB_EXTEND = 12;
    public const SB_LF = 13;
    public const SB_SCONTINUE = 14;
    public const SB_COUNT = 15;
    public const LB_UNKNOWN = 0;
    public const LB_AMBIGUOUS = 1;
    public const LB_ALPHABETIC = 2;
    public const LB_BREAK_BOTH = 3;
    public const LB_BREAK_AFTER = 4;
    public const LB_BREAK_BEFORE = 5;
    public const LB_MANDATORY_BREAK = 6;
    public const LB_CONTINGENT_BREAK = 7;
    public const LB_CLOSE_PUNCTUATION = 8;
    public const LB_COMBINING_MARK = 9;
    public const LB_CARRIAGE_RETURN = 10;
    public const LB_EXCLAMATION = 11;
    public const LB_GLUE = 12;
    public const LB_HYPHEN = 13;
    public const LB_IDEOGRAPHIC = 14;
    public const LB_INSEPARABLE = 15;
    public const LB_INSEPERABLE = 15;
    public const LB_INFIX_NUMERIC = 16;
    public const LB_LINE_FEED = 17;
    public const LB_NONSTARTER = 18;
    public const LB_NUMERIC = 19;
    public const LB_OPEN_PUNCTUATION = 20;
    public const LB_POSTFIX_NUMERIC = 21;
    public const LB_PREFIX_NUMERIC = 22;
    public const LB_QUOTATION = 23;
    public const LB_COMPLEX_CONTEXT = 24;
    public const LB_SURROGATE = 25;
    public const LB_SPACE = 26;
    public const LB_BREAK_SYMBOLS = 27;
    public const LB_ZWSPACE = 28;
    public const LB_NEXT_LINE = 29;
    public const LB_WORD_JOINER = 30;
    public const LB_H2 = 31;
    public const LB_H3 = 32;
    public const LB_JL = 33;
    public const LB_JT = 34;
    public const LB_JV = 35;
    public const LB_CLOSE_PARENTHESIS = 36;
    public const LB_CONDITIONAL_JAPANESE_STARTER = 37;
    public const LB_HEBREW_LETTER = 38;
    public const LB_REGIONAL_INDICATOR = 39;
    public const LB_COUNT = 43;
    public const NT_NONE = 0;
    public const NT_DECIMAL = 1;
    public const NT_DIGIT = 2;
    public const NT_NUMERIC = 3;
    public const NT_COUNT = 4;
    public const HST_NOT_APPLICABLE = 0;
    public const HST_LEADING_JAMO = 1;
    public const HST_VOWEL_JAMO = 2;
    public const HST_TRAILING_JAMO = 3;
    public const HST_LV_SYLLABLE = 4;
    public const HST_LVT_SYLLABLE = 5;
    public const HST_COUNT = 6;
    public const NO_NUMERIC_VALUE = -123456789;
    public const PROPERTY_IDS_UNARY_OPERATOR = 72;
    public const PROPERTY_ID_COMPAT_MATH_START = 73;
    public const PROPERTY_ID_COMPAT_MATH_CONTINUE = 74;

    /**
     * @pure
     */
    public static function hasBinaryProperty(int|string $codepoint, int $property): null|bool
    {
    }

    public static function charAge(int|string $codepoint): null|array
    {
    }

    public static function charDigitValue(int|string $codepoint): null|int
    {
    }

    public static function charDirection(int|string $codepoint): null|int
    {
    }

    public static function charFromName(string $name, int $type = IntlChar::UNICODE_CHAR_NAME): null|int
    {
    }

    public static function charMirror(int|string $codepoint): string|int|null
    {
    }

    public static function charName(int|string $codepoint, int $type = IntlChar::UNICODE_CHAR_NAME): null|string
    {
    }

    public static function charType(int|string $codepoint): null|int
    {
    }

    public static function chr(int|string $codepoint): null|string
    {
    }

    public static function digit(int|string $codepoint, int $base = 10): int|false|null
    {
    }

    public static function enumCharNames(
        int|string $start,
        int|string $end,
        callable $callback,
        int $type = IntlChar::UNICODE_CHAR_NAME,
    ): null|bool {
    }

    public static function enumCharTypes(callable $callback): void
    {
    }

    public static function foldCase(int|string $codepoint, int $options = IntlChar::FOLD_CASE_DEFAULT): string|int|null
    {
    }

    public static function forDigit(int $digit, int $base = 10): int
    {
    }

    public static function getBidiPairedBracket(int|string $codepoint): string|int|null
    {
    }

    public static function getBlockCode(int|string $codepoint): null|int
    {
    }

    public static function getCombiningClass(int|string $codepoint): null|int
    {
    }

    public static function getFC_NFKC_Closure(int|string $codepoint): string|false|null
    {
    }

    public static function getIntPropertyMaxValue(int $property): int
    {
    }

    public static function getIntPropertyMinValue(int $property): int
    {
    }

    public static function getIntPropertyValue(int|string $codepoint, int $property): null|int
    {
    }

    public static function getNumericValue(int|string $codepoint): null|float
    {
    }

    public static function getPropertyEnum(string $alias): int
    {
    }

    public static function getPropertyName(int $property, int $type = IntlChar::LONG_PROPERTY_NAME): string|false
    {
    }

    public static function getPropertyValueEnum(int $property, string $name): int
    {
    }

    public static function getPropertyValueName(
        int $property,
        int $value,
        int $type = IntlChar::LONG_PROPERTY_NAME,
    ): string|false {
    }

    /**
     * @return array{0: int<0, max>, 1: int<0, max>, 2: int<0, max>, 3: int<0, max>}
     */
    public static function getUnicodeVersion(): array
    {
    }

    public static function isalnum(int|string $codepoint): null|bool
    {
    }

    public static function isalpha(int|string $codepoint): null|bool
    {
    }

    public static function isbase(int|string $codepoint): null|bool
    {
    }

    public static function isblank(int|string $codepoint): null|bool
    {
    }

    public static function iscntrl(int|string $codepoint): null|bool
    {
    }

    public static function isdefined(int|string $codepoint): null|bool
    {
    }

    public static function isdigit(int|string $codepoint): null|bool
    {
    }

    public static function isgraph(int|string $codepoint): null|bool
    {
    }

    public static function isIDIgnorable(int|string $codepoint): null|bool
    {
    }

    public static function isIDPart(int|string $codepoint): null|bool
    {
    }

    public static function isIDStart(int|string $codepoint): null|bool
    {
    }

    public static function isISOControl(int|string $codepoint): null|bool
    {
    }

    public static function isJavaIDPart(int|string $codepoint): null|bool
    {
    }

    public static function isJavaIDStart(int|string $codepoint): null|bool
    {
    }

    public static function isJavaSpaceChar(int|string $codepoint): null|bool
    {
    }

    public static function islower(int|string $codepoint): null|bool
    {
    }

    public static function isMirrored(int|string $codepoint): null|bool
    {
    }

    public static function isprint(int|string $codepoint): null|bool
    {
    }

    public static function ispunct(int|string $codepoint): null|bool
    {
    }

    public static function isspace(int|string $codepoint): null|bool
    {
    }

    public static function istitle(int|string $codepoint): null|bool
    {
    }

    public static function isUAlphabetic(int|string $codepoint): null|bool
    {
    }

    public static function isULowercase(int|string $codepoint): null|bool
    {
    }

    public static function isupper(int|string $codepoint): null|bool
    {
    }

    public static function isUUppercase(int|string $codepoint): null|bool
    {
    }

    public static function isUWhiteSpace(int|string $codepoint): null|bool
    {
    }

    public static function isWhitespace(int|string $codepoint): null|bool
    {
    }

    public static function isxdigit(int|string $codepoint): null|bool
    {
    }

    public static function ord(int|string $character): null|int
    {
    }

    public static function tolower(int|string $codepoint): string|int|null
    {
    }

    public static function totitle(int|string $codepoint): string|int|null
    {
    }

    public static function toupper(int|string $codepoint): string|int|null
    {
    }
}

class IntlDatePatternGenerator
{
    public function __construct(null|string $locale = null) {}

    public static function create(null|string $locale = null): null|IntlDatePatternGenerator
    {
    }

    public function getBestPattern(string $skeleton): string|false
    {
    }
}
