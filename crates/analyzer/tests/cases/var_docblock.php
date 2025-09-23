<?php
/**
 * @var string $title
 * @var string $body
 * @var list<string> $items
 */
?>
<html>
<head>
    <title><?= $title ?></title>
</head>
<body>
    <div>
        <?= $body ?>
        <ul>
            <?php foreach ($items as $item): ?>
                <li><?= $item ?></li>
            <?php endforeach; ?>
        </ul>
    </div>
</body>
