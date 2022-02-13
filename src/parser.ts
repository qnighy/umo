export function tokenize(text: string): string[] {
  const tokens: string[] = [];
  let i = 0;
  while (i < text.length) {
    const c = text[i++];
    if (/\s/.test(c)) continue;
    tokens.push(c);
  }
  return tokens;
}
