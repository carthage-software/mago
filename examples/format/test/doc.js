[
  [
    "<?php",
    hardline,
    hardline,
    [
      group([
        "fromArray",
        group(
          [
            "(",
            indent([
              softline,
              group(
                group([
                  "Vec\\map",
                  group([
                    "(",
                    indent([
                      softline,
                      [
                        group(
                          group([
                            "Vec\\chunk",
                            group([
                              "(",
                              indent([
                                softline,
                                [
                                  group(
                                    group([
                                      "$this",
                                      "->",
                                      "toArray",
                                      ["(", ")"],
                                    ]),
                                  ),
                                  ",",
                                  line,
                                ],
                                group("$size"),
                              ]),
                              ifBreak(",", ""),
                              softline,
                              ")",
                            ]),
                          ]),
                        ),
                        ",",
                        line,
                      ],
                      [
                        [
                          "/**\n     * @param array<T> $chunk\n     *\n     * @return static<T>\n     */",
                          breakParent,
                          hardline,
                        ],
                        group(
                          group([
                            "fn",
                            ["(", group("$chunk"), ")"],
                            ifBreak(indent(line), " "),
                            "=>",
                            " ",
                            group([
                              "static",
                              "::",
                              "fromArray",
                              group([
                                "(",
                                indent([softline, group("$chunk")]),
                                ifBreak(",", ""),
                                softline,
                                ")",
                              ]),
                            ]),
                          ]),
                        ),
                      ],
                    ]),
                    ifBreak(",", ""),
                    softline,
                    ")",
                  ]),
                ]),
              ),
            ]),
            ifBreak(",", ""),
            softline,
            ")",
          ],
          { shouldBreak: true },
        ),
      ]),
      ";",
    ],
  ],
  hardline,
];
