<?php

// When an array argument has source line breaks, the parent call's arguments
// should still expand properly across multiple lines.
// The same expression written on a single line should produce the same parent expansion.

// Multi-line array in printf
printf('<a href="%1$s" class="%2$s">%3$s</a>', esc_url(add_query_arg(array(
    'tab' => $slug,
), admin_url('site-health.php'))), $current_tab === $slug ? 'active' : '', esc_html($label));

// Same expression, single-line array
printf('<a href="%1$s" class="%2$s">%3$s</a>', esc_url(add_query_arg(array('tab' => $slug), admin_url('site-health.php'))), $current_tab === $slug ? 'active' : '', esc_html($label));

// Multi-line array in static method call
$wrapper = Html::rawElement('div', array_merge(['class' => [
    'mw-toplinks-content',
    'mw-collapsible-content',
]], $langAttributes), $content);

// Same expression, single-line array
$wrapper = Html::rawElement('div', array_merge(['class' => ['mw-toplinks-content', 'mw-collapsible-content']], $langAttributes), $content);

// Multi-line array in nested function call
$driverChain->addDriver(new XmlDriver(new SymfonyFileLocator([
    __DIR__ . '/../Tests/Resources/orm' => 'Symfony\\Bridge\\Doctrine\\Tests\\Fixtures',
], '.orm.xml'), '.orm.xml', true), 'Symfony\\Bridge\\Doctrine\\Tests\\Fixtures');

// Same expression, single-line array
$driverChain->addDriver(new XmlDriver(new SymfonyFileLocator([__DIR__ . '/../Tests/Resources/orm' => 'Symfony\\Bridge\\Doctrine\\Tests\\Fixtures'], '.orm.xml'), '.orm.xml', true), 'Symfony\\Bridge\\Doctrine\\Tests\\Fixtures');

// Multi-line array in static method call
self::assertSame(trim(Neon::encode([
    'parameters' => [
        'ignoreErrors' => [
            [
                'message' => "#^Escape Regex with file \\# ~ ' \\(\\)$#",
                'count' => 1,
                'path' => 'Testfile',
            ],
        ],
    ],
], Neon::BLOCK)), trim($this->getOutputContent()));

// Same expression, single-line array
self::assertSame(trim(Neon::encode(['parameters' => ['ignoreErrors' => [['message' => "#^Escape Regex with file \\# ~ ' \\(\\)$#", 'count' => 1, 'path' => 'Testfile']]]], Neon::BLOCK)), trim($this->getOutputContent()));
