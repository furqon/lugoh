import type { AnalyzeRequest, AnalyzeResponse } from './types'

const API_BASE = '/api'

export async function analyzeText(req: AnalyzeRequest): Promise<AnalyzeResponse> {
  const res = await fetch(API_BASE + '/analyze', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })

  if (!res.ok) {
    const text = await res.text()
    throw new Error(`Server error (${res.status}): ${text}`)
  }

  return res.json()
}

export async function healthCheck(): Promise<any> {
  const res = await fetch(API_BASE + '/health')
  return res.json()
}
