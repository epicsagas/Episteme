# Proxy

## Essence
Proxy is a structural design pattern that substitutes a stand-in object for the real service, controlling access to it while presenting the same interface to the client. The proxy intercepts calls, performs additional work such as lazy initialization, access checks, logging, or caching, and then forwards the request to the actual service object. Clients interact with the proxy as if it were the real thing, unaware that an intermediary manages lifecycle and cross-cutting concerns.

## Motivation
A photo-sharing application displays thumbnails in a grid but loads full-resolution images only when a user clicks one. Storing hundreds of high-resolution bitmaps in memory would exhaust the device's RAM before the user scrolls past the first screen. Eagerly loading every image at startup is also unacceptable because the user may never view most of them. The application needs a way to keep the image list responsive while deferring expensive loading until it is truly necessary.

A virtual proxy wraps each image entry. The proxy implements the same `Image` interface as the real bitmap but holds only the thumbnail data initially. When the UI requests the full-resolution pixels, the proxy checks whether the real image has been loaded; if not, it fetches it from disk or network, caches it internally, and returns the pixels. Subsequent calls return the cached image immediately. The UI code is unaware that the first call triggered a background download because the proxy's interface is identical to the real image's interface.

## Participants
The service interface declares the operations both the real service and the proxy must support. The real service class implements business logic and holds the expensive resource. The proxy class implements the same interface, holds a reference to the real service (created lazily or injected), and manages access through additional logic before or after delegation. The client uses the service interface and cannot distinguish the proxy from the real object. Several proxy variants address different concerns: a virtual proxy defers expensive creation, a protection proxy enforces access control, a remote proxy handles network communication, a caching proxy stores previous results, and a smart reference tracks usage for reference counting or write-through persistence.

## Application

**Use when:**
- Creating the real object is expensive and should be deferred until its first use (virtual proxy)
- Access to the real object must be restricted based on permissions or roles (protection proxy)
- The real object lives in a separate address space or on a remote server (remote proxy)
- Repeated requests can be served from a cache to avoid redundant computation or network calls (caching proxy)
- Additional bookkeeping is needed, such as reference counting or change tracking (smart reference)

**Prefer alternatives when:**
- No lifecycle management, access control, or cross-cutting concern warrants an intermediary (direct use is simpler)
- The goal is to present a different interface rather than control access to the same interface (Adapter applies)
- A simpler interface over a complex subsystem is needed rather than a stand-in with the same interface (Facade applies)

## Consequences
Proxy introduces a layer that can manage lazy initialization, access control, logging, and caching transparently, which keeps these concerns out of both the client and the real service. The real service may never be instantiated if the proxy determines the request can be handled from cache or denied by policy. However, every proxy adds a class and a level of indirection, which can complicate debugging when call stacks pass through multiple proxy layers. The response time for the first request to a virtual proxy includes the full initialization cost of the real service, which can cause unexpected latency spikes if not anticipated. Protection proxies must be carefully designed to avoid security holes where a determined client bypasses the proxy to reach the real object directly.

## Relations
Proxy and Decorator share the same structural pattern of wrapping an object behind a shared interface, but their intents differ: Decorator adds visible behavior to the object, while Proxy controls access and manages lifecycle. Adapter changes the interface, whereas Proxy preserves it. Facade presents a new simplified interface over a subsystem, while Proxy presents the same interface as the real object. Remote proxy is conceptually related to Broker in distributed systems because both hide network communication details. Flyweight and Proxy both manage object creation efficiency, but Flyweight shares state across many objects while Proxy controls access to a single expensive object.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
