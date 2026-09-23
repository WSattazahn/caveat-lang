// Omitted payloads mean the conventional empty object; explicit null stays null.
export function payloadJson(event) {
  return JSON.stringify(Object.hasOwn(event, 'payload') ? event.payload : {});
}
