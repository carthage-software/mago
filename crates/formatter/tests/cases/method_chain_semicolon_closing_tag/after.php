<?php

$total = $this
    ->getEntityManager()
    ->getRepository(Game::class)
    ->createQueryBuilder('game')
    ->select('COUNT(game.id)')
    ->getQuery()
    ->getSingleScalarResult()
;
?>

<p>Total: <?= $total ?></p>
