import tseslint from "typescript-eslint"
import angular from "@angular-eslint/eslint-plugin"
import angularTemplate from "@angular-eslint/eslint-plugin-template"
import angularTemplateParser from "@angular-eslint/template-parser"
import prettier from "eslint-config-prettier"

export default tseslint.config(
    {
        files: ["web/**/*.ts"],
        extends: [tseslint.configs.recommended],
        plugins: {
            "@angular-eslint": angular,
        },
        processor: angularTemplate.processors["extract-inline-html"],
        rules: {
            ...angular.configs.recommended.rules,
        },
    },
    {
        files: ["web/**/*.html"],
        languageOptions: {
            parser: angularTemplateParser,
        },
        plugins: {
            "@angular-eslint/template": angularTemplate,
        },
        rules: {
            ...angularTemplate.configs.recommended.rules,
        },
    },
    prettier,
)
