import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

// Apply dark mode based on system preference
if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
  document.documentElement.classList.add('dark')
}

// Register service worker for PWA offline support.
// Only in production — during dev, the SW would cache HMR responses
// which causes stale assets after hot reloads.
if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').then(
      (registration) => {
        console.log('AGOS SW registered:', registration.scope)
      },
      (error) => {
        console.warn('AGOS SW registration failed:', error)
      }
    )
  })
}

createApp(App).mount('#app')
