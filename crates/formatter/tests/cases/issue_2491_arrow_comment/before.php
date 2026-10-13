<?php

$f = fn ($x) => /* commentaire */
    $x->nom();

$inline = fn ($x) => /* commentaire */ $x->nom();

$ownLine = fn ($x) =>
    /* commentaire */
    $x->nom();

$line = fn ($x) => // commentaire
    $x->nom();

$binary = fn ($x) => /* commentaire */
    $x->nom() . ' suffixe';

$wrapped = fn ($x) => (
    /* commentaire */ $x->nom()
);
