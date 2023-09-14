namespace rs volo.redis

struct GetRequest {
    1: required string key;
}

struct TrGetRequest {
    1: required string key;
    2: required string id;
}

struct GetResponse {
    1: optional string value;
}

struct SetRequest {
    1: required string key;
    2: required string value;
    3: optional i32 ttl;
}

struct TrSetRequest {
    1: required string key;
    2: required string value;
    3: required string id;
    4: optional i32 ttl;
}

struct SetResponse {
    1: required bool success;
}

struct DelRequest {
    1: required string key;
}

struct TrDelRequest {
    1: required string key;
    2: required string id;
}

struct DelResponse {
    1: required bool success;
}

typedef string PingResponse

struct SubscribeRequest {
    1: required list<string> topics;
}

struct SubscribeResponse {
    1: required list<i32> offsets;
}

struct UnsubscribeRequest {
    1: required list<string> topics;
}

struct UnsubscribeResponse {
    1: required bool success;
}

struct PublishRequest {
    1: required string topic;
    2: required string data;
}

struct PublishResponse {
    1: required bool success;
}

struct Message {
    1: required string topic;
    2: required string data;
}

struct PollRequest {
    1: required list<string> topics;
    2: required list<i32> offsets;
}

struct PollResponse {
    1: required list<Message> messages;
    2: required list<i32> offsets;
}

typedef string MultiResponse

struct ExecResponse {
    1: required bool valid;
    2: required list<string> results;
}

struct WatchRequest {
    1: required string key;
    2: required string id;
}


typedef string ExecRequest
typedef bool BoolResponse

service Redis {
    GetResponse get(1: GetRequest request);
    SetResponse set(1: SetRequest request);
    SetResponse sync(1: SetRequest request);
    DelResponse del(1: DelRequest request);
    DelResponse sync_del(1: DelRequest request);
    PingResponse ping();
    SubscribeResponse subscribe(1: SubscribeRequest request);
    UnsubscribeResponse unsubscribe(1: UnsubscribeRequest request);
    PublishResponse publish(1: PublishRequest request);
    PollResponse poll(1: PollRequest request);
    MultiResponse multi();
    ExecResponse exec(1: ExecRequest request);
    BoolResponse watch(1: WatchRequest request);
    BoolResponse trget(1: TrGetRequest request);
    BoolResponse trset(1: TrSetRequest request);
    BoolResponse trdel(1: TrDelRequest request);
}

service Proxy {
    GetResponse get(1: GetRequest request);
    SetResponse set(1: SetRequest request);
    DelResponse del(1: DelRequest request);
    PingResponse ping();
}

