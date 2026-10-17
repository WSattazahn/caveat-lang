// Resolve hook for measure.mjs: the package's ./node.mjs, as imported by
// lib/authoring.mjs, becomes a loader that keeps one runtime, as the Worker does.
export async function resolve(specifier, context, next) {
  if (specifier === './node.mjs' && context.parentURL?.endsWith('/caveat-lang/lib/authoring.mjs')) {
    return { url: new URL('./measure-runtime.mjs', import.meta.url).href, shortCircuit: true };
  }
  return next(specifier, context);
}
