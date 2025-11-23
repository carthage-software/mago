<?php

$a = "Hello, World!"
    |> strtoupper(?)
    |> trim(...)
    |> str_split(?, 2)
    |> array_map(strrev(...), ?)
    |> implode(" - ", ?);

echo $a; // Output: "OL - LE - ,W - OR - LD"
