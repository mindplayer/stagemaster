use super::Session;
use crate::output_control::{Request, Snapshot};
impl Session {
    pub(crate) fn output_request(&mut self, request: Request) -> Result<Snapshot, String> {
        let uncontrolled = self.document.as_ref().map_or(
            0,
            stagemaster_project::Document::uncontrolled_intensity_fixtures,
        );
        self.output_control.request(request, uncontrolled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stagemaster_project::{Document, EditCommand};
    #[test]
    fn control_is_not_history_and_old_engine_or_duplicate_requests_cannot_restore_lights() {
        let mut session = Session::default();
        session.replace(Document::new("总控").unwrap(), None);
        let before = session.document.clone();
        let generation = session.generation;
        let initial = session.output_request(Request::Snapshot).unwrap();
        session
            .output_request(Request::Set {
                epoch: initial.epoch,
                serial: 1,
                percent: 40,
                blackout: true,
            })
            .unwrap();
        assert!(
            session
                .output_request(Request::Set {
                    epoch: initial.epoch,
                    serial: 1,
                    percent: 100,
                    blackout: false
                })
                .is_err()
        );
        assert!(
            session
                .output_request(Request::Set {
                    epoch: initial.epoch,
                    serial: 2,
                    percent: 101,
                    blackout: false
                })
                .is_err()
        );
        assert!(session.output_control.master().blackout());
        assert_eq!(session.document, before);
        assert!(session.undo.is_empty());
        assert_eq!(session.generation, generation);
        session
            .edit(
                generation,
                EditCommand::SetInfo {
                    name: "编辑后".into(),
                    description: String::new(),
                },
            )
            .unwrap();
        session.history(session.generation, false).unwrap();
        assert!(session.output_control.master().blackout());
        session.replace(Document::new("另一工程").unwrap(), None);
        assert!(
            session
                .output_request(Request::Set {
                    epoch: initial.epoch,
                    serial: 3,
                    percent: 0,
                    blackout: true
                })
                .is_err()
        );
        let reset = session.output_request(Request::Snapshot).unwrap();
        assert_eq!(reset.percent, 100);
        assert!(!reset.blackout);
        assert_eq!(reset.serial, 0);
    }
}
