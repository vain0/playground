/* eslint-disable no-undef */

module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  plugins: [
    "@typescript-eslint",
  ],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/eslint-recommended",
    "plugin:@typescript-eslint/recommended",
  ],
  rules: {
    // 複数行の配列やオブジェクトリテラルの各項目の後ろのカンマは必須 (なければ警告)
    "comma-dangle": [
      "warn",
      "always-multiline",
    ],
    // セミコロンは使わない (あったら警告)
    "semi": [
      "warn",
      "never",
    ],
    // アロー関数を含めてすべての関数の結果型の注釈を必須にするルール
    "@typescript-eslint/explicit-function-return-type": 0,
    // export される関数の型注釈を必須にするルール
    "@typescript-eslint/explicit-module-boundary-types": 0,
    // 未使用の変数宣言を禁止するルール
    // JSX で使う import React を unused として検出する不具合があるようなので、無効化する。
    "@typescript-eslint/no-unused-vars": 0,
    // エラーではなく警告にする。
    "prefer-const": "warn",
  },
}
