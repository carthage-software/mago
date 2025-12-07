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
        <?php echo $level1; /* hrrr */ ?>
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
