// SPDX-License-Identifier: WTFPL

export default {
  async fetch(request) {
    const url = new URL(request.url);
    if (!url.pathname.startsWith("/hc/") || !url.pathname.endsWith(".jpg"))
      return new Response("");
    return await fetch(`https://hc.fly.dev${url.pathname}?${(new Date()).getUTCDate()}${crypto.randomUUID()}`, {cf:{cacheTtl:0}});
  },
};
