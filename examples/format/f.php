<?php

$loader = new DoctrineChoiceLoader(
    $this->om,
    $this->class,
    $this->idReader,
    $this->objectLoader,
);
