import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Q-SHIELD | Post-Quantum Satellite Transmission & Authentication',
  description: 'Research Platform for Post-Quantum Secure Satellite Image Transmission, Fragmentation & Scheduling',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}
