/**
 * Proxies /api/* requests to the Rust backend.
 * Set BACKEND_URL in Vercel project settings (e.g. https://your-api.railway.app).
 */
export default async function handler(request) {
  const backend = process.env.BACKEND_URL?.replace(/\/$/, '');

  if (!backend) {
    return Response.json(
      { error: 'BACKEND_URL is not configured in Vercel environment variables' },
      { status: 503 },
    );
  }

  const incoming = new URL(request.url);
  const target = `${backend}${incoming.pathname}${incoming.search}`;

  const headers = new Headers();
  const contentType = request.headers.get('content-type');
  if (contentType) {
    headers.set('content-type', contentType);
  }

  const upstream = await fetch(target, {
    method: request.method,
    headers,
    body: ['GET', 'HEAD'].includes(request.method) ? undefined : request.body,
  });

  return new Response(upstream.body, {
    status: upstream.status,
    headers: upstream.headers,
  });
}

export const config = {
  runtime: 'edge',
};
