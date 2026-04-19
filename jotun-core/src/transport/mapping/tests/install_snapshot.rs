use super::strategies;
use crate::records::install_snapshot::{InstallSnapshotResponse, RequestInstallSnapshot};
use crate::transport::mapping::ConvertError;
use crate::transport::protobuf as proto;
use proptest::prelude::*;
use prost::Message as _;

fn valid_proto_request_install_snapshot() -> impl Strategy<Value = proto::RequestInstallSnapshot> {
    strategies::request_install_snapshot().prop_map(proto::RequestInstallSnapshot::from)
}

fn valid_proto_install_snapshot_response() -> impl Strategy<Value = proto::InstallSnapshotResponse> {
    strategies::install_snapshot_response().prop_map(proto::InstallSnapshotResponse::from)
}

proptest! {
    #[test]
    fn request_install_snapshot_roundtrip(r in strategies::request_install_snapshot()) {
        let round: RequestInstallSnapshot =
            proto::RequestInstallSnapshot::from(r.clone()).try_into().unwrap();
        prop_assert_eq!(r, round);
    }

    #[test]
    fn request_install_snapshot_wire_roundtrip(r in strategies::request_install_snapshot()) {
        let bytes = proto::RequestInstallSnapshot::from(r.clone()).encode_to_vec();
        let round: RequestInstallSnapshot =
            proto::RequestInstallSnapshot::decode(bytes.as_slice()).unwrap().try_into().unwrap();
        prop_assert_eq!(r, round);
    }

    #[test]
    fn install_snapshot_response_roundtrip(r in strategies::install_snapshot_response()) {
        let round: InstallSnapshotResponse =
            proto::InstallSnapshotResponse::from(r).try_into().unwrap();
        prop_assert_eq!(r, round);
    }

    #[test]
    fn install_snapshot_response_wire_roundtrip(r in strategies::install_snapshot_response()) {
        let bytes = proto::InstallSnapshotResponse::from(r).encode_to_vec();
        let round: InstallSnapshotResponse =
            proto::InstallSnapshotResponse::decode(bytes.as_slice()).unwrap().try_into().unwrap();
        prop_assert_eq!(r, round);
    }

    #[test]
    fn request_install_snapshot_zero_leader_rejected(
        mut p in valid_proto_request_install_snapshot(),
    ) {
        p.leader_id = 0;
        prop_assert_eq!(RequestInstallSnapshot::try_from(p), Err(ConvertError::ZeroNodeId));
    }

    #[test]
    fn request_install_snapshot_missing_last_included_rejected(
        mut p in valid_proto_request_install_snapshot(),
    ) {
        p.last_included = None;
        prop_assert_eq!(
            RequestInstallSnapshot::try_from(p),
            Err(ConvertError::MissingField("RequestInstallSnapshot.last_included"))
        );
    }

    #[test]
    fn install_snapshot_response_missing_last_included_rejected(
        mut p in valid_proto_install_snapshot_response(),
    ) {
        p.last_included = None;
        prop_assert_eq!(
            InstallSnapshotResponse::try_from(p),
            Err(ConvertError::MissingField("InstallSnapshotResponse.last_included"))
        );
    }
}
