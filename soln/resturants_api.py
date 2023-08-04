from flask import Flask, jsonify, request
import pymongo

app = Flask(__name__)

# MongoDB Configuration
MONGO_HOST = 'localhost'
MONGO_PORT = 27017
MONGO_DB = 'restaurants'
MONGO_COLLECTION = 'restaurants_collection'

# Establish MongoDB connection
mongo_client = pymongo.MongoClient(MONGO_HOST, MONGO_PORT)
database = mongo_client[MONGO_DB]
collection = database[MONGO_COLLECTION]


@app.route('/top_restaurants/<int:limit>', methods=['GET'])
def get_top_restaurants(limit):
    try:
        pipeline = [
            {"$match": {"$expr": {"$gte": [{"$size": "$grades"}, limit]}}},
            {"$unwind": "$grades"},
            {"$group": {"_id": "$restaurant_id", "name": {"$first": "$name"}, "avgScore": {"$avg": "$grades.score"}}},
            {"$sort": {"avgScore": -1}},
            {"$limit": limit},
        ]

        result = list(collection.aggregate(pipeline))
        return jsonify(result)

    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/update_grade', methods=['PUT'])
def update_restaurant_grade():
    try:
        data = request.get_json()
        restaurant_name = data.get('name')
        new_grade = data.get('new_grade')

        if not restaurant_name or not new_grade:
            return jsonify({'message': 'Invalid data provided in request body.'}), 400

        result = collection.update_one({'name': restaurant_name, 'grades.grade': 'A'}, {'$set': {'grades.$.grade': new_grade}})

        if result.modified_count > 0:
            return jsonify({'message': 'Update successful.'}), 200
        else:
            return jsonify({'message': 'No matching document found. Update failed.'}), 404

    except Exception as e:
        return jsonify({'error': str(e)}), 500


@app.route('/restaurants_borough_cuisine/<str:borough>/<str:cuisine>/<int:limit>', methods=['GET'])
def get_restaurants_borough_cusiene(borough, cuisine, limit):
    try:
        query = {}
        if borough:
            query['borough'] = borough
        if cuisine:
            query['cuisine'] = cuisine

        pipeline = [{"$match": query}, {"$project": {"_id": 0, "name": 1, "borough": 1, "cuisine": 1}}, {"$limit": limit}]

        result = list(collection.aggregate(pipeline))
        return jsonify(result)

    except Exception as e:
        return jsonify({'error': str(e)}), 500


if __name__ == '__main__':
    app.run(debug=True)
