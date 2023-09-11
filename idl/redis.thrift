namespace rs volo.redis

struct GetRequest {
    1: required string key;
}

struct GetResponse {
    1: optional string value;
}

struct SetRequest {
    1: required string key;
    2: required string value;
    3: optional i32 ttl;
}

struct SetResponse {
    1: required bool success;
}

struct DelRequest {
    1: required string key;
}

struct DelResponse {
    1: required bool success;
}

typedef string PingResponse

struct SubscribeRequest {
    1: required list<string> topics;
}

struct Message {
    1: required string topic;
    2: required string data;
}

struct SubscribeResponse {
    1: required list<Message> messages;
}

struct PublishRequest {
    1: required string topic;
    2: required string data;
}

struct PublishResponse {
    1: required bool success;
}

service Redis {
    GetResponse get(1: GetRequest request);
    SetResponse set(1: SetRequest request);
    DelResponse del(1: DelRequest request);
    PingResponse ping();
    SubscribeResponse subscribe(1: SubscribeRequest request);
    PublishResponse publish(1: PublishRequest request);
}



