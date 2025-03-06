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
    ['الاسم',    'العمر', 'المدينة', 'المهنة'],
    ['أحمد',    '30',    'الرياض',  'مهندس'],
    ['فاطمة',   '25',    'جدة',     'طبيبة'],
    ['علي',     '35',    'الدمام',  'محاسب'],
    ['ليلى',    '28',    'مكة',     'معلمة'],
    ['خالد',    '40',    'المدينة', 'مدير'],
    ['سارة',    '22',    'تبوك',    'طالبة'],
    ['يوسف',    '32',    'حائل',    'مبرمج'],
    ['نورة',    '29',    'أبها',    'مصممة'],
    ['عبدالله', '38',    'جازان',   'محامٍ'],
];

$arr = [
    ['Name',    'Age', 'City',     'Occupation'],
    ['John',    '30',  'New York', 'Engineer'],
    ['Jane',    '25',  'London',   'Doctor'],
    ['Mike',    '35',  'Paris',    'Accountant'],
    ['Emily',   '28',  'Tokyo',    'Teacher'],
    ['David',   '40',  'Sydney',   'Manager'],
    ['Sarah',   '22',  'Toronto',  'Student'],
    ['Robert',  '32',  'Berlin',   'Programmer'],
    ['Jessica', '29',  'Rome',     'Designer'],
    ['William', '38',  'Madrid',   'Lawyer'],
];

$arr = [
    ['姓名', '年龄', '城市', '职业'],
    ['李明', '30',   '北京', '工程师'],
    ['王芳', '25',   '上海', '医生'],
    ['张伟', '35',   '广州', '会计'],
    ['刘丽', '28',   '深圳', '教师'],
    ['陈刚', '40',   '杭州', '经理'],
    ['杨梅', '22',   '成都', '学生'],
    ['赵强', '32',   '南京', '程序员'],
    ['周红', '29',   '武汉', '设计师'],
    ['孙军', '38',   '西安', '律师'],
];

$arr = [
    ['名前', '年齢', '都市',   '職業'],
    ['田中', '30',   '東京',   'エンジニア'],
    ['佐藤', '25',   '大阪',   '医者'],
    ['鈴木', '35',   '名古屋', '会計士'],
    ['高橋', '28',   '福岡',   '教師'],
    ['伊藤', '40',   '札幌',   'マネージャー'],
    ['渡辺', '22',   '仙台',   '学生'],
    ['加藤', '32',   '広島',   'プログラマー'],
    ['山本', '29',   '京都',   'デザイナー'],
    ['中村', '38',   '横浜',   '弁護士'],
];

$arr = [
    ['이름',   '나이', '도시', '직업'],
    ['김민수', '30',   '서울', '엔지니어'],
    ['박지영', '25',   '부산', '의사'],
    ['최성훈', '35',   '대구', '회계사'],
    ['정수진', '28',   '인천', '교사'],
    ['강동현', '40',   '광주', '매니저'],
    ['송하늘', '22',   '대전', '학생'],
    ['윤재혁', '32',   '울산', '프로그래머'],
    ['신혜리', '29',   '수원', '디자이너'],
    ['한정우', '38',   '창원', '변호사'],
];

$arr = [
    ['Имя',       'Возраст', 'Город',           'Профессия'],
    ['Иван',      '30',      'Москва',          'Инженер'],
    ['Елена',     '25',      'Санкт-Петербург', 'Врач'],
    ['Сергей',    '35',      'Новосибирск',     'Бухгалтер'],
    ['Ольга',     '28',      'Екатеринбург',    'Учитель'],
    ['Дмитрий',   '40',      'Нижний Новгород', 'Менеджер'],
    ['Анастасия', '22',      'Казань',          'Студент'],
    ['Алексей',   '32',      'Челябинск',       'Программист'],
    ['Юлия',      '29',      'Самара',          'Дизайнер'],
    ['Андрей',    '38',      'Омск',            'Юрист'],
];

$arr = [
    ['Nom',    'Âge', 'Ville',       'Profession'],
    ['Jean',   '30',  'Paris',       'Ingénieur'],
    ['Marie',  '25',  'Lyon',        'Médecin'],
    ['Pierre', '35',  'Marseille',   'Comptable'],
    ['Sophie', '28',  'Toulouse',    'Professeur'],
    ['Luc',    '40',  'Nice',        'Directeur'],
    ['Claire', '22',  'Nantes',      'Étudiante'],
    ['Paul',   '32',  'Strasbourg',  'Programmeur'],
    ['Alice',  '29',  'Montpellier', 'Designer'],
    ['Michel', '38',  'Bordeaux',    'Avocat'],
];

$arr = [
    ['Nombre', 'Edad', 'Ciudad',    'Profesión'],
    ['Juan',   '30',   'Madrid',    'Ingeniero'],
    ['María',  '25',   'Barcelona', 'Médico'],
    ['Pedro',  '35',   'Valencia',  'Contable'],
    ['Laura',  '28',   'Sevilla',   'Profesor'],
    ['Carlos', '40',   'Bilbao',    'Gerente'],
    ['Ana',    '22',   'Zaragoza',  'Estudiante'],
    ['Luis',   '32',   'Málaga',    'Programador'],
    ['Elena',  '29',   'Murcia',    'Diseñador'],
    ['Javier', '38',   'Palma',     'Abogado'],
];

$arr = [
    ['ชื่อ',     'อายุ', 'เมือง',      'อาชีพ'],
    ['สมชาย',  '30',  'กรุงเทพ',    'วิศวกร'],
    ['สมหญิง',  '25',  'เชียงใหม่',   'แพทย์'],
    ['สมศักดิ์',  '35',  'ภูเก็ต',      'นักบัญชี'],
    ['สมศรี',   '28',  'ขอนแก่น',    'ครู'],
    ['สมบูรณ์',  '40',  'ชลบุรี',      'ผู้จัดการ'],
    ['สมใจ',   '22',  'นครราชสีมา', 'นักเรียน'],
    ['สมหวัง',  '32',  'สุราษฎร์ธานี', 'โปรแกรมเมอร์'],
    ['สมนึก',   '29',  'อุบลราชธานี', 'นักออกแบบ'],
    ['สมหมาย', '38',  'หาดใหญ่',    'ทนายความ'],
];

$arr = [
    ['Tên',  'Tuổi', 'Thành phố',   'Nghề nghiệp'],
    ['Tuấn', '30',   'Hà Nội',      'Kỹ sư'],
    ['Lan',  '25',   'Hồ Chí Minh', 'Bác sĩ'],
    ['Hùng', '35',   'Đà Nẵng',     'Kế toán'],
    ['Mai',  '28',   'Hải Phòng',   'Giáo viên'],
    ['Nam',  '40',   'Cần Thơ',     'Quản lý'],
    ['Hoa',  '22',   'Biên Hòa',    'Sinh viên'],
    ['Dũng', '32',   'Huế',         'Lập trình viên'],
    ['Thảo', '29',   'Nha Trang',   'Nhà thiết kế'],
    ['Long', '38',   'Vũng Tàu',    'Luật sư'],
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
