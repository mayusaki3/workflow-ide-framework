//! Consumer-visible application lifecycle events.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationEvent {
    Starting,
    Started,
    CloseRequested,
    Closing,
}

pub type ApplicationEventHandler = Box<dyn FnMut(ApplicationEvent)>;

#[derive(Default)]
pub struct ApplicationEventDispatcher {
    handlers: Vec<ApplicationEventHandler>,
    started_emitted: bool,
}

impl ApplicationEventDispatcher {
    pub fn subscribe(&mut self, handler: impl FnMut(ApplicationEvent) + 'static) {
        self.handlers.push(Box::new(handler));
    }
    pub fn emit(&mut self, event: ApplicationEvent) {
        if event == ApplicationEvent::Started {
            if self.started_emitted {
                return;
            }
            self.started_emitted = true;
        }
        for handler in &mut self.handlers {
            handler(event);
        }
    }
}
