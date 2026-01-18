/**
 * @file Blazing simple language
 * @author Ata Sesli <atasesli05@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "kiro",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
