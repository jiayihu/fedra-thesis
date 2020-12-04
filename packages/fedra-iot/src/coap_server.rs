use alloc::vec::Vec;
use coap_lite::{
    CoapRequest, CoapResponse, MessageClass, MessageType, ObserveOption, Packet,
    ResponseType as Status, Subject,
};
use rtt_target::rprintln;
use smoltcp::wire::IpEndpoint;

pub struct CoapServer {
    message_id: u16, // TODO: should be Wrapping?
    pub subject: Subject<IpEndpoint>,
}

impl CoapServer {
    pub fn handle_request<F>(&mut self, mut handler: F)
    where
        F: FnMut(CoapRequest<IpEndpoint>) -> Option<CoapResponse>,
    {
        super::network::handle_request(|data, endpoint| {
            let packet = Packet::from_bytes(data).unwrap();
            let mut request = CoapRequest::from_packet(packet, endpoint);

            if request.message.header.get_type() == MessageType::Acknowledgement {
                self.subject.acknowledge(&request);
                return None;
            }

            let observe = request.get_observe();

            if let Some(observe_flag) = observe {
                match observe_flag {
                    ObserveOption::Register => {
                        self.subject.register(&request);

                        if let Some(response) = request.response.as_mut() {
                            response.set_observe(ObserveOption::Register);
                        }
                    }
                    ObserveOption::Deregister => {
                        self.subject.deregister(&request);
                    }
                }
            }

            let response = handler(request).and_then(|response| {
                return response.message.to_bytes().ok();
            });

            response
        });
    }

    pub fn notify_observers(&mut self, resource_path: &str, payload: Vec<u8>) {
        let message_id = self.gen_message_id();
        let resource = self.subject.get_resource(resource_path);
        let observers = self.subject.get_resource_observers(resource_path);

        if let Some(resource) = resource {
            if let Some(observers) = observers {
                observers.into_iter().for_each(|observer| {
                    let notification = self.create_notification(
                        message_id,
                        observer.token.clone(),
                        resource.sequence,
                        payload.clone(),
                    );

                    self.send_message(observer.endpoint, notification);
                })
            }
        }
    }

    fn create_notification(
        &self,
        message_id: u16,
        token: Vec<u8>,
        sequence: u32,
        payload: Vec<u8>,
    ) -> Packet {
        let mut packet = Packet::new();

        packet.header.set_type(MessageType::Confirmable);
        packet.header.code = MessageClass::Response(Status::Content);
        packet.header.message_id = message_id;
        packet.set_token(token);
        packet.set_observe(sequence.to_be_bytes().to_vec());
        packet.payload = payload;

        packet
    }

    fn gen_message_id(&mut self) -> u16 {
        self.message_id += 1;
        return self.message_id;
    }

    pub fn send_message(&self, _endpoint: IpEndpoint, _message: Packet) {
        rprintln!("Sending the message");
    }
}

impl Default for CoapServer {
    fn default() -> Self {
        CoapServer {
            message_id: 0,
            subject: Subject::default(),
        }
    }
}
