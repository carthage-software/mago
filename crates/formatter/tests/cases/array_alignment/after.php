<?php

show_table([ // This is a comment
    ['Month',     'Premium', 'Revenue'], // This is a comment
    ['January',   '$0.00',   '$0.00'],
    ['February',  '$0.00',   '$0.00'],
    ['March',     '$0.00',   '$0.00'],
    ['April',     '$0.00',   '$0.00'],
    ['May',       '$0.00',   '$0.00'],
    ['June',      '$0.00',   '$0.00'],
    ['July',      '$0.00',   '$0.00'],
    ['August',    '$0.00',   '$0.00'],
    // September ..
    ['September', '$0.00',   '$0.00'],
    /// Weeeee!
    /// Weee!!!!
    /* Weeeee! */
    ['October',   '$0.00',   '$0.00'],
    ['November',  '$0.00',   '$0.00'],
    ['December',  '$0.00',   '$0.00'], // This is a comment
]); // This is a comment

show_table([ // This is a comment
    ['Hello', 11212, 112.1, true,  $bar,  PHP_VERSION],
    ['World', 125.1, 12,    false, $quux, PHP_VERSION_ID],
]);

show_table([[[ // This is a comment
    ['Hello', 11212, 112.1, true,  $bar,  PHP_VERSION],
    ['World', 125.1, 12,    false, $quux, PHP_VERSION_ID],
]]]);

show_table(array( // This is a comment
    array('Month',     'Premium', 'Revenue'), // This is a comment
    array('January',   '$0.00',   '$0.00'),
    array('February',  '$0.00',   '$0.00'),
    array('March',     '$0.00',   '$0.00'),
    array('April',     '$0.00',   '$0.00'),
    array('May',       '$0.00',   '$0.00'),
    array('June',      '$0.00',   '$0.00'),
    array('July',      '$0.00',   '$0.00'),
    array('August',    '$0.00',   '$0.00'),
    array('September', '$0.00',   '$0.00'),
    array('October',   '$0.00',   '$0.00'),
    array('November',  '$0.00',   '$0.00'),
    array('December',  '$0.00',   '$0.00'), // This is a comment
)); // This is a comment

show_table([ // This is a comment
    array('Hello', 11212, 112.1,  true,  $bar,  PHP_VERSION),
    array('World', 125.1, 12,     false, $quux, PHP_VERSION_ID),
    array('!!',    125.1, 123512, false, $bar,  PHP_VERSION_ID),
]);

// This table contains a very long string so it won't be formatted as a table
show_table([ // This is a comment
    array(
        'HelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHello',
        11212,
        112.1,
        true,
        $bar,
        PHP_VERSION,
    ),
    array('World', 125.1, 12, false, $quux, PHP_VERSION_ID),
    array('!!', 125.1, 123512, false, $bar, PHP_VERSION_ID),
]);

// too small for table.
$a = [[1, 2], [3, 4], [5, 6]];

$arr = [
    ['الاسم',  'العمر',         'المدينة', 'المهنة'],
    ['أحمد',  'ثلاثون',         'الرياض',  'مهندس'],
    ['فاطمة', 'خمسة وعشرون',   'جدة',     'طبيبة'],
    ['علي',   'خمسة وثلاثون',   'الدمام',  'محاسب'],
    ['ليلى',  'ثمانية وعشرون', 'مكة',     'معلمة'],
    ['خالد',  'أربعون',        'المدينة', 'مدير'],
    ['سارة',  'اثنان وعشرون',  'تبوك',    'طالبة'],
    ['يوسف',  'اثنان وثلاثون',  'حائل',    'مبرمج'],
    ['نورة',  'تسعة وعشرون',   'أبها',    'مصممة'],
    ['عبدالله',  'ثمانية وثلاثون', 'جازان',   'محامٍ'],
];

$arr2 = [
    ['Name',    'Age',          'City',     'Occupation'],
    ['John',    'thirty',       'New York', 'Engineer'],
    ['Jane',    'twenty-five',  'London',   'Doctor'],
    ['Mike',    'thirty-five',  'Paris',    'Accountant'],
    ['Emily',   'twenty-eight', 'Tokyo',    'Teacher'],
    ['David',   'forty',        'Sydney',   'Manager'],
    ['Sarah',   'twenty-two',   'Toronto',  'Student'],
    ['Robert',  'thirty-two',   'Berlin',   'Programmer'],
    ['Jessica', 'twenty-nine',  'Rome',     'Designer'],
    ['William', 'thirty-eight', 'Madrid',   'Lawyer'],
];

$arr3 = [
    ['姓名', '年龄',   '城市', '职业'],
    ['李明', '三十',   '北京', '工程师'],
    ['王芳', '二十五', '上海', '医生'],
    ['张伟', '三十五', '广州', '会计'],
    ['刘丽', '二十八', '深圳', '教师'],
    ['陈刚', '四十',   '杭州', '经理'],
    ['杨梅', '二十二', '成都', '学生'],
    ['赵强', '三十二', '南京', '程序员'],
    ['周红', '二十九', '武汉', '设计师'],
    ['孙军', '三十八', '西安', '律师'],
];

$arr4 = [
    ['名前', '年齢',   '都市',   '職業'],
    ['田中', '三十',   '東京',   'エンジニア'],
    ['佐藤', '二十五', '大阪',   '医者'],
    ['鈴木', '三十五', '名古屋', '会計士'],
    ['高橋', '二十八', '福岡',   '教師'],
    ['伊藤', '四十',   '札幌',   'マネージャー'],
    ['渡辺', '二十二', '仙台',   '学生'],
    ['加藤', '三十二', '広島',   'プログラマー'],
    ['山本', '二十九', '京都',   'デザイナー'],
    ['中村', '三十八', '横浜',   '弁護士'],
];

$arr5 = [
    ['이름',   '나이',     '도시', '직업'],
    ['김민수', '서른',     '서울', '엔지니어'],
    ['박지영', '스물다섯', '부산', '의사'],
    ['최성훈', '서른다섯', '대구', '회계사'],
    ['정수진', '스물여덟', '인천', '교사'],
    ['강동현', '마흔',     '광주', '매니저'],
    ['송하늘', '스물둘',   '대전', '학생'],
    ['윤재혁', '서른둘',   '울산', '프로그래머'],
    ['신혜리', '스물아홉', '수원', '디자이너'],
    ['한정우', '서른여덟', '창원', '변호사'],
];

$arr6 = [
    ['Имя',       'Возраст',         'Город',           'Профессия'],
    ['Иван',      'тридцать',        'Москва',          'Инженер'],
    ['Елена',     'двадцать пять',   'Санкт-Петербург', 'Врач'],
    ['Сергей',    'тридцать пять',   'Новосибирск',     'Бухгалтер'],
    ['Ольга',     'двадцать восемь', 'Екатеринбург',    'Учитель'],
    ['Дмитрий',   'сорок',           'Нижний Новгород', 'Менеджер'],
    ['Анастасия', 'двадцать два',    'Казань',          'Студент'],
    ['Алексей',   'тридцать два',    'Челябинск',       'Программист'],
    ['Юлия',      'двадцать девять', 'Самара',          'Дизайнер'],
    ['Андрей',    'тридцать восемь', 'Омск',            'Юрист'],
];

$arr7 = [
    ['Nom',    'Âge',         'Ville',       'Profession'],
    ['Jean',   'trente',      'Paris',       'Ingénieur'],
    ['Marie',  'vingt-cinq',  'Lyon',        'Médecin'],
    ['Pierre', 'trente-cinq', 'Marseille',   'Comptable'],
    ['Sophie', 'vingt-huit',  'Toulouse',    'Professeur'],
    ['Luc',    'quarante',    'Nice',        'Directeur'],
    ['Claire', 'vingt-deux',  'Nantes',      'Étudiante'],
    ['Paul',   'trente-deux', 'Strasbourg',  'Programmeur'],
    ['Alice',  'vingt-neuf',  'Montpellier', 'Designer'],
    ['Michel', 'trente-huit', 'Bordeaux',    'Avocat'],
];

$arr8 = [
    ['Nombre', 'Edad',            'Ciudad',    'Profesión'],
    ['Juan',   'treinta',         'Madrid',    'Ingeniero'],
    ['María',  'veinticinco',     'Barcelona', 'Médico'],
    ['Pedro',  'treinta y cinco', 'Valencia',  'Contable'],
    ['Laura',  'veintiocho',      'Sevilla',   'Profesor'],
    ['Carlos', 'cuarenta',        'Bilbao',    'Gerente'],
    ['Ana',    'veintidós',       'Zaragoza',  'Estudiante'],
    ['Luis',   'treinta y dos',   'Málaga',    'Programador'],
    ['Elena',  'veintinueve',     'Murcia',    'Diseñador'],
    ['Javier', 'treinta y ocho',  'Palma',     'Abogado'],
];

$arr9 = [
    ['ชื่อ',     'อายุ',      'เมือง',      'อาชีพ'],
    ['สมชาย',  'สามสิบ',    'กรุงเทพ',    'วิศวกร'],
    ['สมหญิง',  'ยี่สิบห้า',    'เชียงใหม่',   'แพทย์'],
    ['สมศักดิ์',  'สามสิบห้า',  'ภูเก็ต',      'นักบัญชี'],
    ['สมศรี',   'ยี่สิบแปด',   'ขอนแก่น',    'ครู'],
    ['สมบูรณ์',  'สี่สิบ',      'ชลบุรี',      'ผู้จัดการ'],
    ['สมใจ',   'ยี่สิบสอง',   'นครราชสีมา', 'นักเรียน'],
    ['สมหวัง',  'สามสิบสอง', 'สุราษฎร์ธานี', 'โปรแกรมเมอร์'],
    ['สมนึก',   'ยี่สิบเก้า',   'อุบลราชธานี', 'นักออกแบบ'],
    ['สมหมาย', 'สามสิบแปด', 'หาดใหญ่',    'ทนายความ'],
];

$arr10 = [
    ['Tên',  'Tuổi',          'Thành phố',   'Nghề nghiệp'],
    ['Tuấn', 'ba mươi',       'Hà Nội',      'Kỹ sư'],
    ['Lan',  'hai mươi lăm',  'Hồ Chí Minh', 'Bác sĩ'],
    ['Hùng', 'ba mươi lăm',   'Đà Nẵng',     'Kế toán'],
    ['Mai',  'hai mươi tám',  'Hải Phòng',   'Giáo viên'],
    ['Nam',  'bốn mươi',      'Cần Thơ',     'Quản lý'],
    ['Hoa',  'hai mươi hai',  'Biên Hòa',    'Sinh viên'],
    ['Dũng', 'ba mươi hai',   'Huế',         'Lập trình viên'],
    ['Thảo', 'hai mươi chín', 'Nha Trang',   'Nhà thiết kế'],
    ['Long', 'ba mươi tám',   'Vũng Tàu',    'Luật sư'],
];

function _trailing_comments(): iterable
{
    yield [(object) [
        'name' => 'saif',
        'articles' => [
            ['title' => 'biz', 'content' => 'foo', 'upvotes' => 4], // 'likes' replaced by 'upvotes'
            ['title' => 'biz', 'content' => 'foo', 'upvotes' => 4], // 'likes' replaced by 'upvotes'
        ],
    ]];

    yield [(object) [
        'name' => 'saif',
        'articles' => [
            ['title' => 'biz', 'content' => 'foo', 'upvotes' => 4], // 'likes' replaced by 'upvotes'
            ['title' => 'biz', 'content' => 'foo', 'upvotes' => 4], // 'likes' replaced by 'upvotes'
        ],
    ]];
}

$arr = [
    [Language::Thai,         render_thai(),       Alignment::left()],
    [Language::Arabic,       render_arabic(),     Alignment::right()],
    [Language::English,      render_english(),    Alignment::center()],
    [Language::French,       render_french(),     Alignment::left()],
    [Language::Spanish,      render_spanish(),    Alignment::right()],
    [Language::Russian,      render_russian(),    Alignment::center()],
    [Language::Japanese,     render_japanese(),   Alignment::left()],
    [Language::Korean,       render_korean(),     Alignment::right()],
    [Language::Vietnamese,   render_vietnamese(), Alignment::center()],
    [Language::Chinese,      render_chinese(),    Alignment::left()],
    [Language::German,       render_german(),     Alignment::right()],
    [Language::Tunisian,     render_tunisian(),   Alignment::center()],
    [Language::Italian,      render_italian(),    Alignment::left()],
    [Language::Portuguese,   render_portuguese(), Alignment::right()],
];

$data = [
    ['Month',          'Premium', 'Revenue'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
    [HtmlValue::any(), '$0.00',   '$0.00'],
];

$data = [
    ['Endorsement Type',      'Endorsement Status', 'Endorsement Creation Date', 'Gross Written Premium'],
    ['0 - Bind',              'Issued',             '08/19/2021',                '$450.00'],
    [HtmlValue::any()],
    ['1 - Change of Address', 'Issued',             '05/02/2021',                '$10.00'],
    [HtmlValue::any()],
];
