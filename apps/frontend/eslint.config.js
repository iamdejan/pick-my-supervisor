import js from "@eslint/js";
import solid from "eslint-plugin-solid/configs/typescript";
import * as tsParser from "@typescript-eslint/parser";
import tseslint from "typescript-eslint";
import globals from "globals";
import eslintPluginPrettierRecommended from "eslint-plugin-prettier/recommended";

export default [
  {
    ignores: ["node_modules/", "dist/"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    ...solid,
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: "tsconfig.json",
      },
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  eslintPluginPrettierRecommended,
];
