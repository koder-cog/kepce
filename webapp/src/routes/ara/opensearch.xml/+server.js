export async function GET({ url }) {
  const origin = url.origin;
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/">
  <ShortName>Kepçe Ara</ShortName>
  <Description>Gizlilik odaklı, açık kaynaklı meta arama motoru</Description>
  <InputEncoding>UTF-8</InputEncoding>
  <Image width="16" height="16" type="image/x-icon">${origin}/favicon.ico</Image>
  <Url type="text/html" template="${origin}/ara?q={searchTerms}"/>
  <Url type="application/x-suggestions+json" template="${origin}/ara/api/suggest?q={searchTerms}"/>
</OpenSearchDescription>`;

  return new Response(xml, {
    headers: {
      "Content-Type": "application/opensearchdescription+xml; charset=utf-8",
      "Cache-Control": "public, max-age=86400",
    },
  });
}
