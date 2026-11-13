import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
    plugins: [
        react(),
        babel({ presets: [reactCompilerPreset()] }),
        tailwindcss(),
    ],
    server: {
        proxy: {
            // Frontend dev server forwards API calls to the axum backend.
            '/api': 'http://127.0.0.1:8080',
            '/health': 'http://127.0.0.1:8080',
        },
    },
})
