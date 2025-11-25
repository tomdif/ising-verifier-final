import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Nova Ising Dashboard',
  description: 'Decentralized Ising optimization marketplace',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
