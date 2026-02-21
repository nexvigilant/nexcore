import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.nexvigilant.sos',
  appName: 'SOS',
  webDir: 'dist',
  server: {
    // In dev, point to trunk serve
    // url: 'http://localhost:8080',
    cleartext: true, // Allow HTTP in dev
  },
  ios: {
    contentInset: 'automatic',
    preferredContentMode: 'mobile',
  },
  android: {
    allowMixedContent: true, // Dev only — remove for prod
  },
};

export default config;
