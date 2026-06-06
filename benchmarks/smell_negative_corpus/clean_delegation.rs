// Guards against: Middle Man FP, Feature Envy FP.
// A logging decorator adds cross-cutting concern (logging) around each call.
// Delegation to `inner` is the point — it's a decorator, not mindless forwarding.

/// Simple logger trait for demonstration.
pub trait Logger {
    fn log(&self, message: &str);
}

/// A service request payload.
#[derive(Debug)]
pub struct Request {
    pub id: u64,
    pub action: String,
}

/// A service response payload.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Core service trait that concrete implementations must satisfy.
pub trait Service {
    fn execute(&self, req: Request) -> Response;
    fn health_check(&self) -> Response;
    fn get_status(&self, id: u64) -> Response;
    fn cancel(&self, id: u64) -> Response;
}

/// A logging decorator that wraps any Service and logs each call.
///
/// Each method adds logging before and after delegation — the delegation
/// itself is intentional, not accidental.
pub struct LoggingService<T: Service> {
    inner: T,
    logger: Box<dyn Logger>,
}

impl<T: Service> LoggingService<T> {
    pub fn new(inner: T, logger: Box<dyn Logger>) -> Self {
        Self { inner, logger }
    }
}

impl<T: Service> Service for LoggingService<T> {
    fn execute(&self, req: Request) -> Response {
        self.logger.log(&format!("execute request id={}", req.id));
        let response = self.inner.execute(req);
        self.logger.log(&format!("execute response status={}", response.status));
        response
    }

    fn health_check(&self) -> Response {
        self.logger.log("health_check called");
        let response = self.inner.health_check();
        self.logger.log(&format!("health_check status={}", response.status));
        response
    }

    fn get_status(&self, id: u64) -> Response {
        self.logger.log(&format!("get_status id={}", id));
        let response = self.inner.get_status(id);
        self.logger.log(&format!("get_status response={}", response.status));
        response
    }

    fn cancel(&self, id: u64) -> Response {
        self.logger.log(&format!("cancel id={}", id));
        let response = self.inner.cancel(id);
        self.logger.log(&format!("cancel response={}", response.status));
        response
    }
}

/// A simple concrete service for testing.
pub struct EchoService;

impl Service for EchoService {
    fn execute(&self, req: Request) -> Response {
        Response { status: 200, body: req.action }
    }

    fn health_check(&self) -> Response {
        Response { status: 200, body: "ok".into() }
    }

    fn get_status(&self, id: u64) -> Response {
        Response { status: 200, body: format!("status of {}", id) }
    }

    fn cancel(&self, id: u64) -> Response {
        Response { status: 204, body: format!("cancelled {}", id) }
    }
}

/// A logger that collects messages into a vector for testing.
pub struct VecLogger {
    pub messages: std::cell::RefCell<Vec<String>>,
}

impl VecLogger {
    pub fn new() -> Self {
        Self { messages: std::cell::RefCell::new(Vec::new()) }
    }
}

impl Logger for VecLogger {
    fn log(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_service_logs_execute() {
        let logger = Box::new(VecLogger::new());
        let logging = LoggingService::new(EchoService, logger);
        let req = Request { id: 1, action: "hello".into() };
        let resp = logging.execute(req);
        assert_eq!(resp.status, 200);
        assert_eq!(logging.logger.messages.borrow().len(), 2);
    }

    #[test]
    fn logging_service_logs_health_check() {
        let logger = Box::new(VecLogger::new());
        let logging = LoggingService::new(EchoService, logger);
        let resp = logging.health_check();
        assert_eq!(resp.status, 200);
        assert_eq!(logging.logger.messages.borrow().len(), 2);
    }
}
