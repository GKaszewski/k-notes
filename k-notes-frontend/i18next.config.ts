import { defineConfig } from 'i18next-cli';

export default defineConfig({
  locales: [
    "en",
    "pl",
    "es",
    "de",
    "fr"
  ],
  extract: {
    input: "src/**/*.{js,jsx,ts,tsx}",
    output: "public/locales/{{language}}/{{namespace}}.json"
  }
});