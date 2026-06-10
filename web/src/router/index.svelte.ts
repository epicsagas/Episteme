export type Route =
  | { page: 'dashboard' }
  | { page: 'explorer' }
  | { page: 'entity'; id: string; from?: 'explorer' | 'ontology' }
  | { page: 'ontology' };

let current: Route = $state({ page: 'dashboard' });

export function getCurrentRoute(): Route {
  return current;
}

export function navigate(route: Route) {
  current = route;
}
