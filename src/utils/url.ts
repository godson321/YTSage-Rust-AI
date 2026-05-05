export function extractUrlsFromText(text: string): string[] {
  if (!text) {
    return [];
  }

  const matches = text.match(/https?:\/\/\S+/gi) ?? [];
  const seen = new Set<string>();
  const urls: string[] = [];

  for (const match of matches) {
    const cleanUrl = match.trim().replace(/[.,;]+$/u, "");
    if (cleanUrl && !seen.has(cleanUrl)) {
      seen.add(cleanUrl);
      urls.push(cleanUrl);
    }
  }

  return urls;
}
