use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

pub mod arguments;
pub mod assignment;
pub mod binaryish;
pub mod control_structure;
pub mod expression;
pub mod mixed;
pub mod statement;
pub mod string;

#[test]
pub fn test_shebang() {
    let code = indoc! {r#"
        #!/usr/bin/env php
        <?php

        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        #!/usr/bin/env php
        <?php

        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_inline_html() {
    let code = indoc! {r#"
        <?php

        $a = 1;
        $b = 2;
        $c = $a + $b;

        ?>
        <a>a + b = <?= $c ?></a>
        <?php

        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        <?php

        $a = 1;
        $b = 2;
        $c = $a + $b;

        ?>
        <a>a + b = <?= $c ?></a>
        <?php

        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_inline_html_alignment() {
    let code = indoc! {r#"
        <?php if ($foo) { ?>
            <?= strtr($this->trace, [
                '#DD0000' => 'var(--string)',
            '#007700' => 'var(--keyword)',
                '#0000BB' => 'var(--default)',
                '#FF8000' => 'var(--comment)',
            ]) ?>

            <?= strtr($this->trace, [
                    '#DD0000' => 'var(--string)',
                 '#007700' => 'var(--keyword)',
                     '#0000BB' => 'var(--default)',
                    '#FF8000' => 'var(--comment)',
                ]); ?>
        <?php } ?>
    "#};

    let expected = indoc! {r#"
        <?php if ($foo) { ?>
            <?= strtr($this->trace, [
                '#DD0000' => 'var(--string)',
                '#007700' => 'var(--keyword)',
                '#0000BB' => 'var(--default)',
                '#FF8000' => 'var(--comment)',
            ]) ?>

            <?= strtr($this->trace, [
                '#DD0000' => 'var(--string)',
                '#007700' => 'var(--keyword)',
                '#0000BB' => 'var(--default)',
                '#FF8000' => 'var(--comment)',
            ]); ?>
        <?php }
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_inline_php() {
    let code = indoc! {r#"
        <div class="mx-2 mb-1 mt-<?= $marginTop ?>">
            <span class="px-1 bg-<?= $bgColor ?> text-<?= $fgColor ?>"><?= $title ?></span>
            <span class="<?= $title ? 'ml-1' : '' ?>">
                <?= htmlspecialchars($content) ?>
            </span>
        </div>
    "#};

    let expected = indoc! {r#"
        <div class="mx-2 mb-1 mt-<?= $marginTop ?>">
            <span class="px-1 bg-<?= $bgColor ?> text-<?= $fgColor ?>"><?= $title ?></span>
            <span class="<?= $title ? 'ml-1' : '' ?>">
                <?= htmlspecialchars($content) ?>
            </span>
        </div>
    "#};

    test_format(code, expected, FormatSettings { print_width: 120, ..FormatSettings::default() });
}

#[test]
pub fn test_inline_echo_statements() {
    let code = indoc! {r#"
        <?php if ($foo) { ?>
            <div id="trace-html-<?= $prefix . '-' . $i; ?>" class="trace-code sf-toggle-content">
            <?= strtr($this->fileExcerpt($trace['file'], $trace['line'], 5), [
                '#DD0000' => 'var(--highlight-string)',
                '#007700' => 'var(--highlight-keyword)',
                '#0000BB' => 'var(--highlight-default)',
                '#FF8000' => 'var(--highlight-comment)',
            ]); ?>
            </div>
        <?php } ?>
    "#};

    let expected = indoc! {r#"
        <?php if ($foo) { ?>
            <div id="trace-html-<?= $prefix . '-' . $i; ?>" class="trace-code sf-toggle-content">
            <?= strtr($this->fileExcerpt($trace['file'], $trace['line'], 5), [
                '#DD0000' => 'var(--highlight-string)',
                '#007700' => 'var(--highlight-keyword)',
                '#0000BB' => 'var(--highlight-default)',
                '#FF8000' => 'var(--highlight-comment)',
            ]); ?>
            </div>
        <?php }
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_html_template() {
    let code = indoc! {r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Extremely Nested PHP Template</title>
        </head>
        <body>
            <h1>Welcome to the Jungle!</h1>
            <?php
            $level1 = "Level 1";
          ?>
            <div class="level1">
                <?php echo $level1; /* hrrr */?>
                <?php

          // This is a comment
                $level2 = "Level 2";
                    // Another comment
              ?>
                <p>
                    <?php echo $level2;?>
                    <ul>
                        <?php
                        $items = ['item1', 'item2', 'item3'];
                        foreach ($items as $item):
                      ?>
                        <li>
                            <?php echo $item;?>
                            <?php
                            $level3 = "Level 3";
                          ?>
                            <span class="level3">
                                <?php echo $level3;?>
                                <?php if (true):?>
                                    <div class="level4">
                                        <?php
                                        $level4 = "Level 4";
                                        echo $level4;
                                      ?>
                                        <?php for ($i = 0; $i < 3; $i++):?>
                                            <p>
                                                <?php
                                                $level5 = "Level 5";
                                                echo $level5. " - ". $i;
                                              ?>
                                            </p>
                                        <?php endfor;?>
                                    </div>
                                <?php endif;?>
                            </span>
                        </li>
                        <?php endforeach;?>
                    </ul>
                </p>
            </div>
        </body>
        </html>
    "#};

    let expected = indoc! {r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Extremely Nested PHP Template</title>
        </head>
        <body>
            <h1>Welcome to the Jungle!</h1>
            <?php
            $level1 = 'Level 1';
            ?>
            <div class="level1">
                <?php echo $level1 /* hrrr */; ?>
                <?php

                // This is a comment
                $level2 = 'Level 2';
                // Another comment
                ?>
                <p>
                    <?php echo $level2; ?>
                    <ul>
                        <?php
                        $items = ['item1', 'item2', 'item3'];
                        foreach ($items as $item): ?>
                        <li>
                            <?php echo $item; ?>
                            <?php
                            $level3 = 'Level 3';
                            ?>
                            <span class="level3">
                                <?php echo $level3; ?>
                                <?php if (true): ?>
                                    <div class="level4">
                                        <?php
                                        $level4 = 'Level 4';
                                        echo $level4;
                                        ?>
                                        <?php for ($i = 0; $i < 3; $i++): ?>
                                            <p>
                                                <?php
                                                $level5 = 'Level 5';
                                                echo $level5 . ' - ' . $i;
                                                ?>
                                            </p>
                                        <?php endfor; ?>
                                    </div>
                                <?php endif; ?>
                            </span>
                        </li>
                        <?php endforeach; ?>
                    </ul>
                </p>
            </div>
        </body>
        </html>
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_mixed_html_php() {
    let code = indoc! {r#"
        <?php if ($condition1): ?>
            <div class="outer-div">
                <?php if ($condition2): ?>
                    <p>Some text inside nested condition.</p>
                    <?php for ($i = 0; $i < 5; $i++): ?>
                        <span>Item <?= $i ?></span>
                        <?php if ($i % 2 == 0): ?>
                            <img src="image<?= $i ?>.jpg" alt="Image <?= $i ?>">
                        <?php else: ?>
                            <?php if ($condition3): ?>
                                <a href="link<?= $i ?>">Link <?= $i ?></a>
                            <?php endif; ?>
                        <?php endif; ?>
                    <?php endfor; ?>
                <?php else: ?>
                    <ul>
                      <?php $items = ["one", "two", "three"]; ?>
                      <?php foreach ($items as $item): ?>
                        <li><?= strtoupper($item) ?></li>
                      <?php endforeach; ?>
                    </ul>
                <?php endif; ?>
            </div>
        <?php endif; ?>
    "#};

    let expected = indoc! {r#"
        <?php if ($condition1): ?>
            <div class="outer-div">
                <?php if ($condition2): ?>
                    <p>Some text inside nested condition.</p>
                    <?php for ($i = 0; $i < 5; $i++): ?>
                        <span>Item <?= $i ?></span>
                        <?php if (($i % 2) == 0): ?>
                            <img src="image<?= $i ?>.jpg" alt="Image <?= $i ?>">
                        <?php else: ?>
                            <?php if ($condition3): ?>
                                <a href="link<?= $i ?>">Link <?= $i ?></a>
                            <?php endif; ?>
                        <?php endif; ?>
                    <?php endfor; ?>
                <?php else: ?>
                    <ul>
                      <?php $items = ['one', 'two', 'three']; ?>
                      <?php foreach ($items as $item): ?>
                        <li><?= strtoupper($item) ?></li>
                      <?php endforeach; ?>
                    </ul>
                <?php endif; ?>
            </div>
        <?php endif;
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_php_85_constant_attributes() {
    let code = indoc! {r#"
        <?php

        #[Deprecated]
        const FOO = 'foo';
    "#};

    let expected = indoc! {r#"
        <?php

        #[Deprecated]
        const FOO = 'foo';
    "#};

    test_format(code, expected, FormatSettings::default())
}
