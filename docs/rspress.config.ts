import * as path from "node:path";
import { defineConfig } from "@rspress/core";

export default defineConfig({
    root: path.join(__dirname, "docs"),
    title: "Waveless",
    themeConfig: {
        socialLinks: [
            {
                icon: "github",
                mode: "link",
                content: "https://github.com/nv0skar/Waveless",
            },
        ],
    },
});
