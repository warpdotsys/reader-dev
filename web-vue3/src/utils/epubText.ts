/** Decode EPUB XHTML as inert text, preserving paragraph breaks and entities.
 * A detached template does not execute scripts or load embedded resources.
 * Never insert this tree into the live document.
 */
export function epubHtmlToText(html: string): string {
  const template = document.createElement('template')
  template.innerHTML = html
  template.content.querySelectorAll(
    'head,title,meta,link,script,style,iframe,object,embed,form,template,noscript',
  ).forEach((element) => element.remove())
  template.content.querySelectorAll('br').forEach((element) => element.replaceWith('\n'))
  template.content.querySelectorAll('p,div,h1,h2,h3,h4,h5,h6,li,tr,blockquote,section,article')
    .forEach((element) => element.append('\n'))
  return (template.content.textContent ?? '').replace(/\r\n?/g, '\n')
    .replace(/[\t \u00a0]+/g, ' ').replace(/ *\n */g, '\n')
    .replace(/\n{3,}/g, '\n\n').trim()
}
